## Upgradeability in ink! contracts

This document describes a method to make your ink! contracts upgradeable.

There are several limitations to this approach, described below. You should
read and understand them before applying this approach to your contracts.

The mechanism used here is based on the [Proxy
pattern](https://blog.openzeppelin.com/proxy-patterns/) used in Ethereum. It
works by having two contracts: the proxy contract, which is the one that other
accounts will interact with, and the internal contract, which contains most of
the smart contract logic. You upgrade your contract by switching the internal
contract referenced by the proxy, making the upgrade process transparent for
the end users of your contract.


## Limitations

You cannot add or change the signature of methods in your contract with this
upgrade mechanism. This is defined in the proxy contract and cannot be changed.
This includes the constructor of the internal contract. It must follow a
specific signature, since it is called by the proxy.

This approach doesn't handle balance transfers. This means that any currency
sent to the proxy won't be forwarded to the internal contract, and any upgrade
examples won't migrate any currency.

When building your internal contracts, you'll need to create methods to expose
its internal state so you can migrate it to a new one. If you don't, you'll
need to reconstruct the state in the new contract indirectly.

There's nothing stopping you from upgrading to a contract that does not match
the signatures required for the proxy to work. Even though we're using rust to
implement the contracts, once they're uploaded to the chain, they're referenced
as code hashes or account ids, so there's no way to enforce a type check when
upgrading.

The `caller`, from the internal contract's point of view, will be the proxy
contract. To remedy this, since most contracts have some authorization
mechanism in place, the proxy passes its caller as an extra argument to the
internal contract.

On each upgrade, the code of the internal contract potentially changes. This
means that the execution cost of each method may change. If other accounts that
interact with your contract, (particularly automated ones) have set execution
limits, they may stop working.

Access to the internal contract should be restricted to the proxy address. We
don't want random folks calling the internal contract after it's no longer the
active one. Everything should go through the proxy, except if we want to allow
for self destructing or augmenting the contract trait somehow. This doesn't
feel very scalable, though.

To upgrade a contract, you need to call the `upgrade` method on the proxy
contract with the code hash of the new internal version. The proxy contract
instantiates the new internal contract instead of receiving an account ID so
that the data migration and reference update is all done in the same
transaction. In previous versions of the contracts pallet you could upload the
contract code and get a code hash by calling `contracts.putCode`. This was
deprecated to solve the storage fees problem in code hashes. Now you can only
upload code as part of a contract deployment. This means that you'll need to
deploy a dummy version of the internal contract you intend to use just to get
the code hash. If your internal contract constructor has any side effects, this
may cause some problems.

## Tutorial

In this section we'll go through the process of adding upgradeability to a
sample contract. The contract will have two methods: one to insert an `i32`
value, and another one to calculate the average of the inserted values. Only
the owner of the contract should be able to insert new values, with `average`
being callable by anyone. In the first version, we'll be using the [arithmetic
mean](https://en.wikipedia.org/wiki/Arithmetic_mean). Afterwards, we'll upgrade
it to use the [median](https://en.wikipedia.org/wiki/Median), with a slight
change in storage to make it more efficient.

### Implementing and deploying the first version

The first step is to implement the contract as if there's no upgradeability:

~~~~rust
#[ink::contract]
mod v1 {
    use ink_prelude::*;

    #[ink(storage)]
    pub struct V1 {
        values: vec::Vec<i32>, // track inserted values
        owner: AccountId,      // track who can call insert
    }

    impl V1 {
        #[ink(constructor)]
        pub fn new() -> Self {
          // store Self::env().caller() as owner
        }

        #[ink(message)]
        pub fn insert(&mut self, value: i32) -> Result<()> {
          // check that caller is owner
          // store value
        }

        #[ink(message)]
        pub fn average(&self) -> Result<i32> {
          // return computed average
        }
    }

    #[cfg(test)]
    mod tests { /* .. */ }
}
~~~~

Next, we change the method signatures, including the constructor's, to receive
an explicit caller. This is necessary because the actual caller will be the
proxy. This is kind of similar to how you use the
[`X-Forwarded-For`](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/X-Forwarded-For)
header in HTTP reverse proxies to pass the IP of the client to the backend.
Here's an example of the `insert` method being changed:

~~~~rust
// original method
#[ink(message)]
pub fn insert(&mut self, value: i32) -> Result<()> {
    self.enforce_owner_call(Self::env().caller())?;

    Ok(self.insert_internal(value))
}

// method with explicit caller
#[ink(message)]
pub fn insert(&mut self, value: i32, caller: AccountId) -> Result<()> {
    self.enforce_owner_call(caller)?;

    Ok(self.insert_internal(value))
}
~~~~


Now we need to ensure that the contract methods are only called by the proxy
contract. To do this, we store the actual caller of the constructor in this
contract's storage, and in every method we add a check that the caller is the
proxy:

~~~~rust
#[ink(storage)]
pub struct V1 {
    values: vec::Vec<i32>, // track inserted values
    owner: AccountId,      // track who can call insert
    proxy: AccountId,      // track the proxy address
}

impl V1 {
    #[ink(constructor)]
    pub fn new(caller: AccountId) -> Self {
      // store caller as owner
      // store Self::env().caller() as proxy
    }

    #[ink(message)]
    pub fn insert(&mut self, value: i32, caller: AccountId) -> Result<()> {
        self.enforce_proxy_call()
        self.enforce_owner_call(caller)?;

        Ok(self.insert_internal(value))
    }

    fn enforce_proxy_call(&self) -> Result<()> {
        if Self::env().caller() != self.proxy {
            Err(Error::NotCalledFromProxy)
        } else {
            Ok(())
        }
    }
~~~~

Now that the contract messages can't be accidentally called without going
through the proxy, we need to add some methods to expose the contract internal
state. These will be used by the constructor of a potential new version, if we
ever decide to upgrade it. We don't need to expose the `proxy` storage entry, but
we need to expose `items`, `nth`, and `owner`:

~~~~rust
#[ink(message)]
pub fn items(&self) -> u32 {
    self.values.len() as u32
}

#[ink(message)]
pub fn nth(&self, idx: u32) -> i32 {
    self.values[idx as usize]
}

#[ink(message)]
pub fn owner(&self) -> AccountId {
    self.owner
}
~~~~

The last step of the contract modification is to add a new constructor. This
constructor would be used to bootstrap this contract from a previous version.
The first version won't ever call this constructor, but we need to have it to
work around ink! type limitations:

~~~~rust
#[ink(constructor)]
pub fn upgrade_from(_v1: AccountId, _caller: AccountId) -> Self {
    panic!("not implemented");
}
~~~~

Now that the internal contract is ready, we need to implement the proxy
contract. This contract will have a constructor, an `upgrade` method, and one
method per message that we want to delegate to the internal contract. The proxy
contract needs to store a reference to the internal contract. Here's the basic
structure:

~~~~rust
#[ink::contract]
mod proxy {
    use ink_prelude::*;
    use v1::V1;

    #[ink(storage)]
    pub struct Proxy {
        backend: V1,
    }

    impl Proxy {
        #[ink(constructor)]
        pub fn new(code_hash: Hash) -> Self {
          // ...
        }

        #[ink(message, payable)]
        pub fn upgrade(&mut self, code_hash: Hash) -> Result<()> {
          // ...
        }

        #[ink(message)]
        pub fn insert(&mut self, value: i32) { /* .. */ }

        #[ink(message)]
        pub fn average(&self) -> i32 { /* .. */ }
    }

    #[cfg(test)]
    mod tests { /* .. */ }
}
~~~~

To implement the delegation methods (`insert` and `average`), we need to pass
all the parameters we received plus the caller:

~~~~rust
#[ink(message)]
pub fn insert(&mut self, value: i32) {
    self.backend.insert(value, Self::env().caller()).unwrap()
}

#[ink(message)]
pub fn average(&self) -> i32 {
    self.backend.average(Self::env().caller()).unwrap()
}
~~~~

In this example, we're unwrapping the internal errors since they only occur if
the internal contract is called from someone without permission. In your
contract, you might want to signal this using `bool`, or even returning the
`Result`. This will depend on the interface you're implementing.

The other two methods, `new` and `upgrade`, should instantiate a new contract
based on the provided code hash and update `self.backend`. When we implemented
the internal contract, we implemented two constructors: `new` and
`upgrade_from`. These should be used in `new` and `upgrade`, respectively:

~~~~rust
#[ink(constructor)]
pub fn new(code_hash: Hash) -> Self {
    let backend = V1::new(Self::env().caller())
        .endowment(Self::env().balance() / 2)
        .code_hash(code_hash)
        .salt_bytes(1i32.to_le_bytes())
        .instantiate()
        .expect("failed at instantiating the internal contract");

    Self { backend: backend }
}

#[ink(message, payable)]
pub fn upgrade(&mut self, code_hash: Hash) -> Result<()> {
    use ink_lang::ToAccountId;

    self.backend = V1::upgrade_from(self.backend.to_account_id(), Self::env().caller())
        .endowment(Self::env().balance() / 2)
        .code_hash(code_hash)
        .salt_bytes(1i32.to_le_bytes())
        .instantiate()
        .map_err(|_e| Error::UpgradeError)?;

    Ok(())
}
~~~~

When calling `upgrade_from`, we pass it the previous contract account ID, so
that the new internal contract can populate itself with the previous version's
data.

While this is enough to have an upgradeable contract, we don't want to allow
anyone to upgrade it, so we need to add authorization checks to the `upgrade`
method. One way we could do this is by keeping track of the proxy contract
owner and only allowing them to upgrade the contract:

~~~~rust
#[ink(constructor)]
pub fn new(code_hash: Hash) -> Self {
    // ...

    Self { backend: backend, owner: Self::env().caller() }
}

#[ink(message, payable)]
pub fn upgrade(&mut self, code_hash: Hash) -> Result<()> {
    use ink_lang::ToAccountId;

    if Self::env().caller() != self.owner {
        return Err(Error::UnauthorizedCaller);
    }

    // ...
}
~~~~

Your use case may call for a different authorization strategy, so you may have
to adapt the authorization code to your needs.

Now that both the proxy and the internal contracts are ready, you can deploy
them. You start by deploying a dummy internal contract so that its code gets
uploaded to the chain. Afterwards, you can deploy the proxy contract with the
internal contract's code hash as its argument. Once the proxy contract is up
and running, you can destroy / reclaim the dummy internal contract (see the
Limitations section for why this is necessary).

Note that the `upgrade` method is marked as payable. When you instantiate a
contract, you need to give it some funds to pay for its storage rent. Having
`upgrade` marked as payable allows you to top up the proxy contract before
instantiating a new internal contract. You may also need to top up the balance
of both the proxy and the internal contracts from time to time.


### Upgrading a deployed contract

After a while, you may decide to upgrade your contract. In our case, we want to
start using the median instead of the arithmetic mean as the average.

Instead of modifying the V1 source code, we create a new contract, V2. This
allows us to have code that references both versions and reflects reality: both
versions of the contract will exist in the blockchain simultaneously. We also
need both implementations so that we can implement the data migration
constructor, which references the older version.

Computing the median requires sorting the values, and our first version didn't
store them in any particular order. We could sort them every time someone calls
the average method, but to improve the performance of this method, we'll be
changing the storage to a sorted array instead. We'll need to sort the existing
values during the migration, and any new value needs to be inserted in the
right position. This will reduce the computation needed in `average`, but
increase it in `insert`.

In this case, the new contract's storage is the same as V1's storage, as are
the guards in the `insert` and `average` methods:

~~~~rust
#[ink(storage)]
pub struct V2 {
    values: vec::Vec<i32>,
    proxy: AccountId,
    owner: AccountId,
}

#[ink(message)]
pub fn insert(&mut self, value: i32, caller: AccountId) -> Result<()> {
    self.enforce_proxy_call()?;
    self.enforce_owner_call(caller)?;

    Ok(self.insert_internal(value))
}

#[ink(message)]
pub fn average(&self) -> Result<i32> {
    self.enforce_proxy_call()?;

    Ok(self.average_internal())
}
~~~~

The actual implementation logic for those two methods is different, though:

~~~~rust
pub fn insert_internal(&mut self, value: i32) {
    let idx = self
        .values
        .binary_search(&value)
        .unwrap_or_else(|x| x);

    self.values.insert(idx, value);
}

pub fn average_internal(&self) -> i32 {
    let n = self.values.len();
    if n == 0 {
        0
    } else if n % 2 == 1 {
        self.values[n / 2]
    } else {
        (self.values[n / 2 - 1] + self.values[n / 2]) / 2
    }
}
~~~~

This version also needs the state exposing methods that we added in V1:
`items`, `nth`, and `owner`. Since we didn't fundamentally change the contract
storage, these are the same, but we could have added or removed storage items.
For example, if `insert` had became available to anyone and not only to the
contract owner, we could have dropped its reference.

The two other methods that need to be implemented are the constructors. The
`upgrade_from` constructor receives a reference to the previous version and
migrates all the data. In this case, we need to make sure that we store the
values sorted. Instead of sorting directly, we can do this by calling
`insert_internal`:

~~~~rust
#[ink(constructor)]
pub fn upgrade_from(v1: AccountId, _caller: AccountId) -> Self {
    use ink_env::call::FromAccountId;
    let previous = V1::from_account_id(v1);

    let mut new = Self {
        values: vec![],
        owner: previous.owner(),
        proxy: Self::env().caller(),
    };

    for i in 0..previous.items() {
        new.insert_internal(previous.nth(i));
    }

    new
}
~~~~

The `new` constructor doesn't deal with any data migrations, so in our case
it's the same as V1's constructor:

~~~~rust
#[ink(constructor)]
pub fn new(caller: AccountId) -> Self {
    Self {
        values: vec![],
        owner: caller,
        proxy: Self::env().caller(),
    }
}
~~~~

With the second version of the internal contract implemented, we can upgrade
our deployed contract. Just like in the first time, we start by deploying a
dummy version of V2, to upload its code to the chain. Once we have the code
hash, we call `upgrade` on the proxy contract. It will deploy a new V2 instance
using the `upgrade_from` constructor and update its internal reference all in
the same transaction to avoid race conditions.

With the new version deployed and running, you can destroy / reclaim the
previous version of the internal contract.


## ink! features that would improve this proposal

The proxy contract's code references the internal contract's type directly.
This doesn't affect functionality, but it feels a bit weird. Ideally we'd use a
trait here, since it can be any contract, but ink! doesn't support [dynamic
trait based contract calling](https://github.com/paritytech/ink/issues/631).
Once that feature is added, this approach can be improved.
