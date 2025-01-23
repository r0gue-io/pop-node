In order to run the integration tests you have to specify what Relay and Pop runtime to use. The first also determines the Asset Hub runtime.
```shell
cargo test --features=<relay>,<pop_runtime>
```
For example:
```shell
cargo test --features=paseo,testnet
```