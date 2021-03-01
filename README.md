## Names of things

contract trait: the public interface of your contracts. The messages that you'll be able to call from the 'chain, ignoring things like upgrades and data migrations and all that infrastructure cruft.

proxy contract: The contract that users will interact with.

internal contract: The contract with the actual code, called only by the proxy contract.


## Things to consider

access to the internal contract should be restricted to the proxy
address. We don't want random folks calling the internal contract
after it's no longer the active one. Everything should go through the
proxy, except if we want to allow for self destructing or augmenting
the contract trait somehow. This doesn't feel very scalable, though.

## Questions

Can we access the storage things from the proxy contract?

Can we set up a catchall selector in the proxy contract to add new calls?

Can we detect that calls to the internal contract are coming from the proxy? investigate `ink_env` `caller`


## Recommendations

add circuit breaker logic anyway.

