# Cross-Chain Messaging with Pop API

This [ink!][ink] contract leverages the [Messaging API][pop-api-messaging] for integration and interoperability with the Polkadot ecosystem and its applications.

It demonstrates how a contract can:
- request state from other chains using Polytope's Interoperable State Machine Protocol (ISMP)
- transact using Polkadot's Cross-Consensus Messaging (XCM).

## Key benefits of using the Pop API

- Simplify cross-chain interactions with powerful high-level interfaces which minimize complexity.
- Usage of ISMP or XCM messaging to satisfy cross-chain use cases.
- Optional callbacks for asynchronous response notifications.

[Learn more how Pop API works.](pop-api)

## Support

Be part of our passionate community of Web3 builders. [Join our Telegram](https://t.me/onpopio)!

Feel free to raise issues if anything is unclear, you have ideas or want to contribute to Pop! Examples using the messaging API are always welcome!

For any questions related to ink! you can also go to [Polkadot Stack Exchange](https://polkadot.stackexchange.com/) or
ask the [ink! community](https://t.me/inkathon/1).

[ink]: https://use.ink
[pop-api]: https://github.com/r0gue-io/pop-node/tree/main/pop-api/
[pop-api-messaging]: https://github.com/r0gue-io/pop-node/tree/main/pop-api-vnext/src/messaging
