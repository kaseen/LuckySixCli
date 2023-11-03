use std::io::Write;
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

#[derive(Debug)]
pub enum LotteryState { READY, STARTED, CALCULATING, DRAWING, CLOSED }
impl From<u8> for LotteryState {
    fn from(num: u8) -> Self {
        match num {
            0 => LotteryState::READY,
            1 => LotteryState::STARTED,
            2 => LotteryState::CALCULATING,
            3 => LotteryState::DRAWING,
            4 => LotteryState::CLOSED,
            _ => panic!("Invalid lottery state")
        }
    }
}

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

    let result = format!(
        "{} {}", 
        parsed_result.unwrap().trim_end_matches('0').trim_end_matches('0'),
        to_denomnination
    );

    Ok(result)
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

    let result = parse_to_denomination(platform_fee_wei, unit).unwrap();

    Ok(result)
}

#[tokio::main]
pub async fn get_lottery_state() -> Result<LotteryState, BoxError> {
    let contract = get_contract_instance().await?;
    let lottery_state: LotteryState = contract.lottery_state().call().await?.into();

    Ok(lottery_state)
}

#[tokio::main]
pub async fn play_ticket(combination: [U256; 6], value: U256) -> Result<[u8; 32], BoxError> {
    let contract = get_contract_instance().await?;
    let tx = contract.play_ticket(combination).value(value);

    match tx.send().await {
        Ok(res) => {
            let tx_hash = res.tx_hash();
            let tx_hash_bytes = tx_hash.to_fixed_bytes();
            return Ok(tx_hash_bytes)
        },
        Err(e) => {
            panic!("{}", e);
        }
    };
}

#[tokio::main]
pub async fn get_drawn_numbers_for_round(n: U256) -> Result<Vec<u8>, BoxError> {
    let contract = get_contract_instance().await?;
    let unpacked = contract.unpack_result_for_round(n).call().await?;

    // TODO: Let it public on blockchain
    let bonus_multiplier = [
        0, 0, 0, 0, 0, 10000, 7500, 5000, 2500, 1000, 500, 300, 200, 150, 100, 90, 80, 70, 60, 50,
        40, 30, 25, 20, 15, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1,
    ];
    
    let mut result = Vec::new();

    for i in 0..5 {
        write!(&mut result, "{} ", unpacked[i])?;
    }
    writeln!(&mut result)?;
    for i in 5..15 {
        let left_mul = bonus_multiplier[i];
        let middle_mul = bonus_multiplier[i + 10];
        let right_mul = bonus_multiplier[i + 20];

        let left_num = unpacked[i];
        let middle_num = unpacked[i + 10];
        let right_num = unpacked[i + 20];

        write!(&mut result, "{left_mul:>5}: {left_num:>2}     ")?;
        write!(&mut result, "{middle_mul:>2}: {middle_num:>2}     ")?;
        write!(&mut result, "{right_mul:>2}: {right_num:>2}\n")?;
    }
    
    Ok(result)
}

#[tokio::main]
pub async fn get_payout_for_ticket(round_number: U256, ticket_index: U256) -> Result<(), BoxError> {
    let contract = get_contract_instance().await?;

    match contract.get_payout_for_ticket(round_number, ticket_index).await {
        Ok(res) => {
            // TODO: Event
            println!("get_payout_for_ticket res: {:?}", res);
            return Ok(())
        },
        Err(e) => {
            panic!("{}", e);
        }
    };
}

#[tokio::main]
pub async fn get_platform_balance() -> Result<String, BoxError> {
    let contract = get_contract_instance().await?;
    let res = contract.platform_balance().call().await?;
    let platform_balance = parse_to_denomination(res, "eth").unwrap();

    Ok(platform_balance)
}

#[tokio::main]
pub async fn get_tickets_for_round(n: U256) -> Result<Vec<u8>, BoxError> {
    let contract = get_contract_instance().await?;
    let res = contract.get_tickets_for_round(n).call().await?;

    let mut result = Vec::new();

    for ticket in res {
        writeln!(&mut result, "{:?} for {}", ticket.combination, parse_to_denomination(ticket.bet, "eth")?)?;
    }

    // Remove last newline character
    result.pop();

    Ok(result)
}
