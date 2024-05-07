# Funding Dev Accounts

As Pop Network uses the Relay chain token as the native token, the dev accounts (alice, bob, etc) are not funded on Pop Network by default.
Therefore, after network launch there needs to be a reserve transfer from the Relay chain to the dev accounts on Pop Network.
This script performs these reserve transfers to fund the dev accounts from the Relay chain.

## Running the script

1. Spin up a Polkadot Network locally with Pop Network running
2. In the `main.rs` file change:
- the `PARA_ID` to the paraId of Pop Network e.g. `9090`
- change the `relay_api` port number to the port number of the Relay chain running on your machine
- change the `pop_api` port number to the port number of Pop Network running on your machince
3. Run the script

```
cargo run
```

### Troubleshooting

Pop Network is ongoing constant new updates and features to its runtime. It is common for the metadata to be out-dated.
Therefore if you run this script and get the following errro:
```
cargo run
   Compiling fund-dev-accounts v0.0.0 (/Users/bruno/src/pop-node/scripts/fund-dev-accounts)
    Finished dev [unoptimized + debuginfo] target(s) in 7.21s
     Running `/Users/bruno/src/pop-node/target/debug/fund-dev-accounts`
Error: Metadata(IncompatibleCodegen)
```

It means you will need to update the metadata.

You can do so by running:
```
subxt codegen --url ws://127.0.0.1:58043 | rustfmt > rococo_interface.rs
subxt codegen --url ws://127.0.0.1:58051 | rustfmt > pop_interface.rs
```

In this example, `58043` is the port number of the Relay chain and `58051` is the port number of Pop Network.

Once the metadata has been updated, re-run your script:
```
cargo run
```

