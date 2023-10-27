use ethers::{
    providers::{ Http, Provider },
    types::{ Address, U256 },
    utils::format_units,
    prelude::abigen
};
use chrono::{
    prelude::DateTime,
    Utc, Local
};

abigen!(LuckySix, "abi.json");

/*
 *  TODO: Config
 *  - Provider
 *  - LuckySix adress
 *  - Set time zone, ex: get_round_info()
 */

async fn get_contract_instance() -> Result<LuckySix<Provider<Http>>, Box<dyn std::error::Error>> {
    let provider_url = std::env::var("HTTP_SEPOLIA").expect("HTTP_SEPOLIA environment variable not found");
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
pub async fn get_round_info() -> Result<(U256, String, bool), Box<dyn std::error::Error>> {
    let contract = get_contract_instance().await?;
    let round_info = contract.round_info().call().await?;

    // Convert to Local timezone
    let datetime = DateTime::<Utc>::from_timestamp(
        round_info.1.as_u64().try_into().unwrap(), 0
    ).expect("Invalid timestamp").with_timezone(&Local);

    let timestamp_string = datetime.format("%d/%m/%Y %H:%M:%S").to_string();

    Ok((round_info.0, timestamp_string, round_info.2))
}

#[tokio::main]
pub async fn get_platform_fee() -> Result<String, Box<dyn std::error::Error>> {
    let contract = get_contract_instance().await?;
    let platform_fee_wei = contract.platform_fee().call().await?;
    let unit = "eth";
    let platform_fee_eth = format!("{} {}", parse_to_denomination(platform_fee_wei, unit), unit);
    Ok(platform_fee_eth)
}
