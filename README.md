## Upgradeability in ink contracts

This document describes a way to add upgradeability to your smart contracts.
There are several limitations to this approach, described below.

The mechanism used here is based on the [Proxy pattern][proxy-pattern] used in
ethereum. In summary, these are the steps you need to take to things:

1. define your contract interface. This cannot be changed via upgradeability
2. implement your contract logic in an "internal contract".
3. implement the "proxy contract", the contract that other accounts will interact with.



## Limitations

You cannot add or change the signature of methods in your contract with this
upgrade mechanism. This is defined in the proxy contract and cannot be changed.

This method doesn't handle balance transfers. This means that any currency sent
to the proxy won't be forwarded to the internal contract, and any upgrade
examples won't migrate any currency.

When building your internal contracts, you'll need to create methods to expose
its internal state so you can migrate it to a new one. If you don't, you'll
need to reconstruct the state in the new contract indirectly.

There's nothing stopping you from upgrading to a broken contract, or rolling
back to a previous contract with a stale state.

The `caller`, from the internal contract's point of view, will be the proxy
contract. To remedy this, since most contracts have some authorization
mechanism in place, the proxy passes its caller as an extra argument to the
internal contract.

On each upgrade, the code of the internal contract potentially changes. This
means that the execution cost of each method may change. If other accounts that
interact with your contract, (particularly automated ones) have set execution
limits, they may stop working.


## Things to consider

Access to the internal contract should be restricted to the proxy address. We
don't want random folks calling the internal contract after it's no longer the
active one. Everything should go through the proxy, except if we want to allow
for self destructing or augmenting the contract trait somehow. This doesn't
feel very scalable, though.

## Questions

Can we set up a catchall selector in the proxy contract to add new calls?


## Limitations

Caller, in the internal contract, will be the proxy contract. This means that
you can't use this to identify who is calling. Additionally, you probably want
to prevent the internal contract from being called directly. This means that
the authorization story would look like this:

~~~~rust

// proxy contract

impl Proxy {
  #[ink(message)]
  fn do_important_stuff(&self) -> Result<()> {
    self.backend.do_important_stuff(Self::env().caller())
  }
}

// internal contract
#[ink::contract]
mod v1 {
  pub struct V1 {
    vip: AccountId
    proxy: AccountId
    // other stuff
  }

  impl V1 {
    #[ink(message)]
    fn do_important_stuff(&self, caller: AccountId) -> Result<()> {
      self.enforce_proxy_call()?;

      if caller != self.vip {
        return Err(Unauthorized);
      }

      // do important stuff

      Ok(())
    }

    fn enforce_proxy_call(&self) -> Result<()> {
      if Self::env().caller() != self.proxy {
        return Err(Error::NotCalledFromProxy)
      } else {
        Ok(())
      }
    }
  }
}

~~~~
