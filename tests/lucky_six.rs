use ethers::{
    abi::Tokenizable,
    contract::abigen,
    middleware::SignerMiddleware,
    providers::{Http, Provider},
    signers::{LocalWallet, Signer},
    types::U256,
    utils::Anvil,
};
use eyre::Result;

abigen!(LuckySix, "luckysix.json");

#[tokio::test]
async fn test() -> Result<()> {
    // Create local Ethereum node
    let anvil = Anvil::new().spawn();

    // Instantiate wallets
    let owner_wallet: LocalWallet = anvil.keys()[0].clone().into();
    let oracle_wallet: LocalWallet = anvil.keys()[1].clone().into();

    // Connect to the local Ethereum network
    let provider = Provider::<Http>::try_from(anvil.endpoint())?
        .interval(std::time::Duration::from_millis(10u64));

    // Instantiate the client with the wallet
    let client = SignerMiddleware::new(provider, owner_wallet.with_chain_id(anvil.chain_id()));
    let client = std::sync::Arc::new(client);

    // Deploy contract
    let contract = LuckySix::deploy(
        client.clone(),
        (
            U256::from(1).into_token(),
            oracle_wallet.clone().address(),
            oracle_wallet.clone().address(),
        ),
    )
    .unwrap()
    .send()
    .await
    .unwrap();

    Ok(())
}
