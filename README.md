# Real Estate Canister
![chain-key-cryptography (1)](https://github.com/Muusii/Rust-101/assets/159922924/68ac4a0a-717f-4f98-a87d-3bf8290b63a9)

## Overview
This real estate canister is a decentralized ICP canister that mainly implements chain-key technology by enabling all owners to have unique public keys to verify them and mitigate the problem of invalid brokers from the market.
Also it provides both query and update functionalities to interface with property data, making it a strong solution for managing properties on the Internet Computer platform.
## Features
+ Create,Update and delete properties.
+ Management of property.
+ Read available properties. 
+ Verify owners of properties.

### Getting Started
- [Start](https://internetcomputer.org/docs/current/developer-docs/getting-started/deploy/local)
- [Developer tools](https://internetcomputer.org/docs/current/developer-docs/getting-started/install)
- [Development Guide](https://internetcomputer.org/docs/current/developer-docs/backend/rust)
- [IC-CDK](https://docs.rs/ic-cdk/latest/ic_cdk)
- [IC-CDK-Macros](https://docs.rs/ic-cdk-macros/latest/ic_cdk_macros)
- [Introduction to Candid](https://internetcomputer.org/docs/current/developer-docs/backend/candid)

## Running Canister
+ Clone my repository:
```
git clone https://github.com/Muusii/Rust-101.git
```
```
cd Rust-101
```
+ View the code:
```
code .
```
+ Start the canister:
```
dfx start --background
```
+ Deploy the canister:
```
npm run gen-deploy
```
+ Update changes:
```
dfx start --background --clean
```
## Contributions
Please feel free to start an issue or send a pull request if you discover any bugs, have ideas, or would like to add new functionality.

## License
This project is licensed under the MIT License.



