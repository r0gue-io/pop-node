![pop-net-banner](https://github.com/r0gue-io/pop-node/assets/60948618/e13ec7e6-1aaf-44bc-8ab3-c7b1b876ea3f)


<div align="center">

[![Twitter URL](https://img.shields.io/twitter/follow/Pop?style=social)](https://twitter.com/pop_web3)
[![Twitter URL](https://img.shields.io/twitter/follow/R0GUE?style=social)](https://twitter.com/gor0gue)
[![Telegram](https://img.shields.io/badge/Telegram-gray?logo=telegram)](https://t.me/PlasmOfficial](https://t.me/Pop_Network)https://t.me/Pop_Network)
[![Build, test and lint](https://github.com/r0gue-io/pop-node/actions/workflows/build.yml/badge.svg)](https://github.com/r0gue-io/pop-node/actions/workflows/build.yml)

</div>

Pop Network makes it easy for smart contract developers to use the Power of Polkadot. Through curated runtime
primitives, smart contract developers can spend less time learning the complexities of Polkadot, and more time buidling
awesome things.

Pop supports Polkadot native contracts (`pallet-contracts`), enabling developers to build with more performant and
secure smart contract languages (such as [ink!](https://use.ink/)).

# Launching Local Network
## Installation
You can install the [Pop CLI](https://github.com/r0gue-io/pop-cli) as follows:
```shell
cargo install --git https://github.com/r0gue-io/pop-cli
```

## Spawn Network
You can spawn a local network as follows:
```shell
pop up parachain -f ./networks/rococo.toml
```
Note: `pop` will automatically source the necessary `polkadot` binaries. Currently, these will have to be built if on a non-linux system.
