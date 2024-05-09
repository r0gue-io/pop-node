# Funding Dev Accounts

As Pop Network uses the Relay chain token as the native token, the dev accounts (alice, bob, etc) are intentionally not
funded on Pop Network by default. Therefore, after network launch there needs to be a reserve transfer from the Relay
chain to these accounts on Pop Network. This script performs these reserve transfers to fund the dev accounts from the
Relay chain.

## Running the script

1. Spin up a Polkadot Network locally with Pop Network running
2. Run the script

```shell
cargo run
```

### Troubleshooting

Pop Network is ongoing constant new updates and features to its runtime. It is common for the metadata to be outdated.
Therefore if you run this script and get the following error:

```shell
cargo run
   Compiling fund-dev-accounts v0.0.0 (/Users/bruno/src/pop-node/scripts/fund-dev-accounts)
    Finished dev [unoptimized + debuginfo] target(s) in 7.21s
     Running `/Users/bruno/src/pop-node/target/debug/fund-dev-accounts`
Error: Metadata(IncompatibleCodegen)
```

It means you will need to update the metadata.

You can do so by running the follow commands within `scripts/fund-dev-accounts`:

```
subxt codegen --url ws://127.0.0.1:8833 | rustfmt > rococo_interface.rs
subxt codegen --url ws://127.0.0.1:9944 | rustfmt > pop_interface.rs
```

Once the metadata has been updated, re-run the script:

```shell
cargo run
```

