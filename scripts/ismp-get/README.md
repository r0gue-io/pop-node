# ISMP Get Testing Script

This is a simple script intended to showcase the flow of performing a GET request
to the ISMP API.

THe script uses a hard-coded storage key to Asset

## Usage

Setup:

```
yarn
```

Ensure ts-node is installed

```
yarn add ts-node typescript @types/node --dev
```

Ensure devnet is launched with `--enable-offchain-indexing=true`

```
pop up parachain -f networks/devnet.toml
```

Run the script:

```
ts-node ismp-get.ts
```

## Acknowledgements

This script was originally inspired and adapted
from [RegionX](https://github.com/RegionX-Labs/RegionX-Node/tree/main/e2e_tests/xc-transfer)