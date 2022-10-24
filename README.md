[//]: # (![Logo]&#40;https://dev-to-uploads.s3.amazonaws.com/uploads/articles/th5xamgrr6se0x5ro4g6.png&#41;)

# Laminar Markets Rust SDK

This repository is the official Rust SDK for Laminar Markets, a high throughput decentralized exchange built on the Aptos Blockchain.

## Installation

Include the following in your `Cargo.toml`
```toml
laminar-sdk = { version = "1.0.0" }
```

## Usage

The following environment variables need to be set:

- `APTOS_NODE_URL`: REST API url of a given Aptos Node.
- `DEX_ACCOUNT_ADDRESS`: Laminar Markets dex account address.

To initialize a `LaminarClient`:
```rust
use laminar_sdk::LaminarClient;
use once_cell::sync::Lazy;

static APTOS_NODE_URL: Lazy<reqwest::Url> = Lazy::new(|| {
    let s = std::env::var("APTOS_NODE_URL").unwrap();
    reqwest::Url::parse(&s).unwrap()
});

static DEX_ACCOUNT: Lazy<AccountAddress> = Lazy::new(|| {
    use aptos_sdk::types::account_address::AccountAddress;
    let a = std::env::var("DEX_ACCOUNT_ADDRESS").unwrap();
    AccountAddress::from_hex_literal(&a).unwrap()
});

pub async fn create_laminar_client() -> Result<LaminarClient, anyhow::Error> {
    let dex_account = &*DEX_ACCOUNT;
    let client = LaminarClient::connect(APTOS_NODE_URL.clone(), dex_account, acc).await?;
    Ok(client)
}
```

## Documentation

[Documentation](https://linktodocumentation)

## Support

For support, please do one of the following:

- Leave a GitHub Issue.
- Contact the Laminar team on [Discord](https://discord.gg/laminar).

## License

[MIT](https://choosealicense.com/licenses/mit/)
