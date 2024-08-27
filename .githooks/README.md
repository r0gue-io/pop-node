# Git Hooks

A pre-push hook which checks formatting of Rust files. Additional checks may be added in the future.

# Prerequisites

The following prerequisites are required:

## Rust Nightly

The nightly version of Rust provides additional formatting benefits over the stable version.

```shell
rustup toolchain install nightly --profile minimal --component rustfmt
```

## Taplo

Formatting of TOML documents is done via `taplo`.
Find more about taplo [here](https://taplo.tamasfe.dev/).

```
cargo install taplo-cli --locked
```

## Zepter

Analyze, fix and format features in your Rust workspace.
Find more about zepter [here](https://github.com/ggwpez/zepter).

```
cargo install zepter -f --locked
```

# Installation

Use the following command in the root directory of the local repository to configure Git to use the hooks:

```shell
git config core.hooksPath .githooks
```
