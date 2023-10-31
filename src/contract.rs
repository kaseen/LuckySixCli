use crate::config::get_config_key;
use chrono::{prelude::DateTime, Local, Utc};
use ethers::{
    utils::{ ConversionError, format_units },
    signers::{LocalWallet, Signer},
    providers::{Http, Provider},
    types::{Address, U256},
    middleware::SignerMiddleware,
    core::types::Chain,
    prelude::abigen,
};

type BoxError = Box<dyn std::error::Error>;
type Client = SignerMiddleware<Provider<Http>, LocalWallet>;
abigen!(LuckySix, "abi.json");

async fn get_provider() -> Result<Provider<Http>, BoxError> {
    let provider_url = get_config_key::<String>("http_sepolia");
    let provider = Provider::<Http>::try_from(provider_url)?;

    Ok(provider)
}

async fn get_wallet() -> Result<LocalWallet, BoxError> {
    let private_key = get_config_key::<String>("private_key");
    let wallet = private_key
        .parse::<LocalWallet>()?
        .with_chain_id(Chain::Sepolia);

    Ok(wallet)
}

async fn get_client() -> Result<Client, BoxError> {
    let provider = get_provider().await?;
    let wallet = get_wallet().await?;
    let client = SignerMiddleware::new(provider, wallet);

    Ok(client)
}

async fn get_contract_instance() -> Result<LuckySix<Client>, BoxError> {
    let client = get_client().await?;

    let lucky_six_address = get_config_key::<String>("lucky_six_address");
    let address: Address = lucky_six_address.parse()?;
    let contract = LuckySix::new(address, client.into());

    Ok(contract)
}

fn parse_to_denomination(input: U256, to_denomnination: &str) -> Result<String, ConversionError> {
    let parsed_result = match to_denomnination {
        "eth" => format_units(input, "Ether"), 
        "gwei" => format_units(input, "Gwei"),
        "wei" => format_units(input, "Wei"),
        _ => Err(ConversionError::UnrecognizedUnits(to_denomnination.to_string()))
    };
    parsed_result
}

#[tokio::main]
pub async fn get_round_info() -> Result<(U256, String, bool), BoxError> {
    let contract = get_contract_instance().await?;
    let round_info = contract.round_info().call().await?;

    // Convert to Local timezone
    let datetime = DateTime::<Utc>::from_timestamp(round_info.1.as_u64().try_into().unwrap(), 0)
        .expect("Invalid timestamp")
        .with_timezone(&Local);

    let timestamp_string = datetime.format("%d/%m/%Y %H:%M:%S").to_string();

    Ok((round_info.0, timestamp_string, round_info.2))
}

#[tokio::main]
pub async fn get_platform_fee() -> Result<String, BoxError> {
    let contract = get_contract_instance().await?;
    let platform_fee_wei = contract.platform_fee().call().await?;
    let unit = "eth";

    let to_denomination = parse_to_denomination(platform_fee_wei, unit).unwrap();
    let parsed_result = to_denomination.trim_end_matches('0').trim_end_matches('0');
    
    let platform_fee_eth = format!("{} {}", parsed_result, unit);

    Ok(platform_fee_eth)
}

#[tokio::main]
pub async fn print_drawn_numbers_for_round(n: U256) -> Result<(), BoxError> {
    let contract = get_contract_instance().await?;
    let result = contract.unpack_result_for_round(n).call().await?;

    // TODO: Let it public on blockchain
    let bonus_multiplier = [
        0, 0, 0, 0, 0, 10000, 7500, 5000, 2500, 1000, 500, 300, 200, 150, 100, 90, 80, 70, 60, 50,
        40, 30, 25, 20, 15, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1,
    ];

    for i in 0..5 {
        print!("{} ", result[i]);
    }
    println!();
    for i in 5..15 {
        let left_mul = bonus_multiplier[i];
        let middle_mul = bonus_multiplier[i + 10];
        let right_mul = bonus_multiplier[i + 20];

        let left_num = result[i];
        let middle_num = result[i + 10];
        let right_num = result[i + 20];

        print!("{left_mul:>5}: {left_num:>2}     ");
        print!("{middle_mul:>2}: {middle_num:>2}     ");
        print!("{right_mul:>2}: {right_num:>2}");
        println!();
    }

    Ok(())
}

#[tokio::main]
pub async fn get_tickets_for_round(n: U256) -> Result<(), BoxError> {
    let contract = get_contract_instance().await?;
    let res = contract.get_tickets_for_round(n).call().await?;

    println!("{:?}", res);

    Ok(())
}
