use ethers::{
    providers::{ Http, Provider },
    types::{ Address, U256 },
    utils::format_units,
    prelude::abigen
};

abigen!(LuckySix, "abi.json");

async fn get_contract_instance() -> Result<LuckySix<Provider<Http>>, Box<dyn std::error::Error>> {
    // TODO: Provider to config
    let provider_url = std::env::var("HTTP_SEPOLIA").expect("HTTP_SEPOLIA environment variable not found");

    // TODO: Address to config
    let lucky_six_address = "0x5d65cff1f21fcfedd194ef7e15e66ae31ab6dcb7";

    let provider = Provider::<Http>::try_from(provider_url)?; 
    let address: Address = lucky_six_address.parse()?;
    let contract = LuckySix::new(address, provider.into());

    Ok(contract)
}

fn parse_to_denomination(input: U256, to_denomnination: &str) -> String {
    let parsed_result = match to_denomnination {
        "eth" => format_units(input, "Ether"),
        "gwei" => format_units(input, "Gwei"),
        "wei" => format_units(input, "Wei"),
        // TODO
        _ => Ok(String::from("Error"))
    };
    String::from(parsed_result.unwrap().trim_end_matches('0').trim_end_matches('.'))
}

#[tokio::main]
pub async fn get_round_info() -> Result<(U256, U256, bool), Box<dyn std::error::Error>> {
    let contract = get_contract_instance().await?;
    let round_info = contract.round_info().call().await?;
    Ok(round_info)
}

#[tokio::main]
pub async fn get_platform_fee() -> Result<String, Box<dyn std::error::Error>> {
    let contract = get_contract_instance().await?;
    let platform_fee_wei = contract.platform_fee().call().await?;
    let unit = "eth";
    let platform_fee_eth = format!("{} {}", parse_to_denomination(platform_fee_wei, unit), unit);
    Ok(platform_fee_eth)
}
