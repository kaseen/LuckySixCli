use ethers::{
    providers::{ Http, Provider },
    prelude::abigen,
    types::Address
};

#[tokio::main]
pub async fn do_thing() -> Result<(), Box<dyn std::error::Error>> {

    let provider_url = std::env::var("HTTP_SEPOLIA").expect("HTTP_SEPOLIA environment variable not found");

    let lucky_six_address = "0x5d65cff1f21fcfedd194ef7e15e66ae31ab6dcb7";

    let provider = Provider::<Http>::try_from(provider_url)?; 
    let address: Address = lucky_six_address.parse()?;
    abigen!(LuckySixContract, "abi.json");
    let contract = LuckySixContract::new(address, provider.into());

    if let Ok(res) = contract.round_info().call().await {
        println!("{:?}", res);
    }

    Ok(())
}
