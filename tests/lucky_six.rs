use ethers::{
    abi::Tokenizable,
    contract::{abigen, EthEvent},
    middleware::SignerMiddleware,
    providers::{Http, Provider, StreamExt},
    signers::{LocalWallet, Signer},
    types::{Bytes, U256},
    utils::{Anvil, AnvilInstance},
};

use eyre::Result;
use std::sync::Arc;
// use luckysix::LotteryState;

abigen!(LuckySix, "luckysix.json");

type Client = SignerMiddleware<Provider<Http>, LocalWallet>;

async fn init_test() -> Result<(
    LuckySix<Client>,
    LuckySix<Client>,
    LuckySix<Client>,
    AnvilInstance,
)> {
    // Create local Ethereum node
    let anvil = Anvil::new().spawn();

    // Instantiate wallets
    let owner_wallet: LocalWallet = anvil.keys()[0].clone().into();
    let oracle_wallet: LocalWallet = anvil.keys()[1].clone().into();
    let user_wallet: LocalWallet = anvil.keys()[2].clone().into();

    // Connect to the local Ethereum network
    let provider = Provider::<Http>::try_from(anvil.endpoint())?
        .interval(std::time::Duration::from_millis(10u64));

    // Instantiate clients with their wallets
    let client_owner = SignerMiddleware::new(
        provider.clone(),
        owner_wallet.clone().with_chain_id(anvil.chain_id()),
    );

    let client_oracle = SignerMiddleware::new(
        provider.clone(),
        oracle_wallet.clone().with_chain_id(anvil.chain_id()),
    );

    let client_user = SignerMiddleware::new(provider, user_wallet.with_chain_id(anvil.chain_id()));

    // Deploy contract
    let contract_instance_owner = LuckySix::deploy(
        Arc::new(client_owner.clone()),
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

    // Create contract instances for oracle and user
    let contract_instance_oracle = LuckySix::new(
        contract_instance_owner.clone().address(),
        Arc::new(client_oracle),
    );
    let contract_instance_user = LuckySix::new(
        contract_instance_owner.clone().address(),
        Arc::new(client_user),
    );

    Ok((
        contract_instance_owner,
        contract_instance_oracle,
        contract_instance_user,
        anvil,
    ))
}

async fn oracle_perform(oracle: &LuckySix<Client>) -> Result<()> {
    oracle.perform_upkeep(Bytes::new()).send().await.unwrap();
    Ok(())
}

#[derive(EthEvent)]
struct RoundStarted {
    num_of_round: U256
}

#[derive(EthEvent)]
struct CountdownStarted {
    num_of_round: U256
}

#[tokio::test]
async fn integration_test() -> Result<()> {
    let (owner_c, oracle_c, user_c, _anvil) = init_test().await.unwrap();

    // Events
    let event_round_started = user_c.event::<RoundStarted>();
    let event_countdown_started = user_c.event::<CountdownStarted>();

    // Assert if RoundStarted emitted
    let mut stream = event_round_started.stream().await?.take(1);
    let _ = oracle_perform(&oracle_c).await;
    let event_received = stream.next().await.unwrap().unwrap();
    assert_eq!(event_received.num_of_round, U256::from(1));

    // Assert if CountdownStarted emitted
    let mut stream = event_countdown_started.stream().await?.take(1);
    let combination = luckysix::convert_to_u256_arr([1, 2, 3, 4, 5, 6]);
    let value = U256::from(15000000000000000_u128); // 0.15eth
    let _ = owner_c.play_ticket(combination).value(value).send().await;
    let event_received = stream.next().await.unwrap().unwrap();
    assert_eq!(event_received.num_of_round, U256::from(1));
    
    // Assert if CountdownStarted emitted
    let mut stream = event_countdown_started.stream().await?.take(1);
    let combination = luckysix::convert_to_u256_arr([1, 2, 3, 4, 5, 6]);
    let value = U256::from(15000000000000000_u128); // 0.15eth
    let _ = owner_c.play_ticket(combination).value(value).send().await;
    let event_received = stream.next().await.unwrap().unwrap();
    assert_eq!(event_received.num_of_round, U256::from(1));

    Ok(())
}
