use ethers::prelude::{k256::ecdsa::SigningKey, *};

pub async fn setup_signer(
    provider: Provider<Http>,
    pkey: String,
) -> (SignerMiddleware<Provider<Http>, Wallet<SigningKey>>, Address) {
    let chain_id = provider
        .get_chainid()
        .await
        .expect("Failed to get chain id.");

    let wallet = pkey
        .parse::<LocalWallet>()
        .expect("Failed to parse wallet")
        .with_chain_id(chain_id.as_u64());
    let address = wallet.address();
    (SignerMiddleware::new(provider, wallet), address)
}

pub fn address(address: &str) -> Address {
    address.parse::<Address>().unwrap()
}

pub fn wei_to_float(input: u128) -> f64 {
    input as f64 / 1_000_000_000_000_000_000.0
}

pub fn get_address_from_pkey(pkey: String) -> Address {
    let wallet = pkey
        .parse::<LocalWallet>()
        .expect("Failed to parse wallet");

    wallet.address()
}