In order to run the integration tests you have to specify what Pop and Relay runtime to use. The latter also determines the Asset Hub runtime.
```shell
cargo test --features=<relay>,<pop_runtime>
```
For example:
```shell
cargo test --features=paseo,testnet
```