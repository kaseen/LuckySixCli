use ethers::{
    providers::{ Http, Provider },
    prelude::abigen,
    types::Address
};

abigen!(LuckySix, "abi.json");

async fn get_contract_instance() -> Result<LuckySix<Provider<Http>>, Box<dyn std::error::Error>> {
    let provider_url = std::env::var("HTTP_SEPOLIA").expect("HTTP_SEPOLIA environment variable not found");

    let lucky_six_address = "0x5d65cff1f21fcfedd194ef7e15e66ae31ab6dcb7";

    let provider = Provider::<Http>::try_from(provider_url)?; 
    let address: Address = lucky_six_address.parse()?;
    let contract = LuckySix::new(address, provider.into());

    Ok(contract)
}

#[tokio::main]
pub async fn do_thing1() -> Result<(), Box<dyn std::error::Error>> {

    let contract = get_contract_instance().await?;

    if let Ok(res) = contract.round_info().call().await {
        println!("{:?}", res);
    }
    
    Ok(())
}

#[tokio::main]
pub async fn do_thing2() -> Result<(), Box<dyn std::error::Error>> {

    let contract = get_contract_instance().await?;

    if let Ok(res) = contract.platform_fee().call().await {
        println!("{:?}", res);
    }

    Ok(())
}
