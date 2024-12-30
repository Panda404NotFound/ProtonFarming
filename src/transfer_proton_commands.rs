use std::process::Command;
use anyhow::Result;
use regex::Regex;
use std::collections::HashMap;
use log::info;
use crate::load_config;

use crate::transfer_balance::get_balance;

 // Функция prepare_deposit подготавливает депозит для фиксированного аккаунта и указанных символов и контрактов.
pub async fn prepare_deposit(symbols: &[(&str, &str)]) -> Result<(String, String), anyhow::Error> {
    let config = load_config();
    let account_name = config.account;

    info!("Start prepare_deposit...");
    let symbols_arg = symbols
        .iter()
        .map(|(sym, contract)| format!("{{\"sym\": \"{}\", \"contract\": \"{}\"}}", sym, contract))
        .collect::<Vec<_>>()
        .join(", ");
 
        let output = Command::new("sh")
        .arg("-c")
        .arg(format!("proton action proton.swaps depositprep '{{\"owner\": \"{account_name}\", \"symbols\": [{}]}}' {account_name}", symbols_arg))
        .output()?;
 
    if output.status.success() {
        info!("Prepare_deposit output: {}", String::from_utf8_lossy(&output.stdout));
        Ok((
            String::from_utf8_lossy(&output.stdout).to_string(),
            String::from_utf8_lossy(&output.stderr).to_string(),
        ))
        
    } else {
        eprintln!(
            "Failed to prepare deposit. Error: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        Err(anyhow::anyhow!("Failed to prepare deposit"))
    }
        
}

pub fn buy_ram_if_needed() -> Result<bool> {
    let config = load_config();
    let account_name = config.account;

    let output = Command::new("sh")
    .arg("-c")
    .arg(format!("proton action eosio buyrambytes '{{\"payer\": \"{account_name}\", \"receiver\": \"{account_name}\", \"bytes\": {}}}' {account_name}", 5000))
    .output()?;

    if !output.status.success() {
        eprintln!("Error executing command: {}", String::from_utf8_lossy(&output.stderr));
        eprintln!("Command output: {}", String::from_utf8_lossy(&output.stdout));
        Ok(false)
    } else {
        info!("buy_ram_if_needed output: {}", String::from_utf8_lossy(&output.stdout));
        Ok(true)
    }
}

// Обертка для prepare_deposit
pub async fn prepare_deposit_wrapper(tokens: &[(&str, &str)]) -> Result<(), anyhow::Error> {
    loop {
        let result = prepare_deposit(tokens).await;
        match result {
            Ok((stdout, _stderr)) => {
                let re = Regex::new(r"has insufficient ram").unwrap();
                if re.is_match(&stdout) {
                    info!("Main.rs: Insufficient RAM detected. Buying RAM...");
                    match buy_ram_if_needed() {
                        Ok(true) => {
                            info!("Main.rs: RAM bought successfully. Retrying prepare_deposit...");
                            continue;
                        }
                        Ok(false) => {
                            eprintln!("Main.rs: Failed to buy RAM.");
                            return Err(anyhow::anyhow!("Failed to buy RAM"));
                        }
                        Err(e) => return Err(e),
                    }
                } else {
                    info!("Main.rs: Prepare deposit successful.");
                    break;
                }
            }
            Err(e) => {
                eprintln!("Main.rs: Failed to prepare deposit. Error: {}", e);
                return Err(e);
            }
        }
    }
    Ok(())
}

 pub async fn transfer_tokens(contract: &str, quantity: &str, to: &str, memo: &str) -> Result<String, anyhow::Error> {
    let config = load_config();
    let account_name = config.account;

    info!("p_c.rs: Start transfer_tokens...");
    let output = Command::new("sh")
    .arg("-c")
    .arg(format!("proton action {} transfer '{{\"from\":\"{account_name}\",\"to\":\"{}\",\"quantity\":\"{}\",\"memo\":\"{}\"}}' {account_name}@active", contract, to, quantity, memo))
    .output()?;

    if !output.status.success() {
        eprintln!("Failed to transfer tokens. Error: {}", String::from_utf8_lossy(&output.stderr));
        return Err(anyhow::anyhow!("Failed to transfer tokens"));
    } else {
        info!("Token transfer successful. Output: {}", String::from_utf8_lossy(&output.stdout));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

pub async fn transfer_excess_snips() -> Result<Option<HashMap<String, String>>, anyhow::Error> {
    let config = load_config();
    let snips_converter_value = config.snips_converter_value;
    let account_name = config.account;

    let mut balances: HashMap<String, String> = get_balance()?;
    let snips_balance_str = balances.get("SNIPS").unwrap_or(&"0.0000".to_string()).to_string();
    let snips_balance = snips_balance_str.parse::<f64>()?;

    // TODO:
    if snips_balance > snips_converter_value {
        let mut quantity = format!("{:.4} SNIPS", snips_balance);
        let mut reduction_factor = 0.9;
        let re = Regex::new(r"overdrawn balance").unwrap();

        info!("Parsing SNIPS in transfer_excess_snips: {}", quantity);

        loop {
            let output = Command::new("sh")
            .arg("-c")
            .arg(format!("proton action snipcoins transfer '{{\"from\": \"{account_name}\", \"to\": \"proton.swaps\", \"quantity\": \"{}\", \"memo\": \"SNIPSXP>XPRUSDC, 5000\"}}' {account_name}@active", quantity))
            .output()?;

            let stdout = String::from_utf8_lossy(&output.stdout);
            info!("Transfer excess SNIPS output: {}", stdout);

            if re.is_match(&stdout) {
                if reduction_factor >= 0.8 {
                    quantity = format!("{:.4} SNIPS", snips_balance * reduction_factor);
                    reduction_factor -= 0.1;
                } else {
                    eprintln!("Failed to transfer excess SNIPS after multiple reductions. Output: {}", stdout);
                    break;
                }
            } else {
                // После успешного перевода токенов, получаем обновленные балансы
                balances = get_balance()?;
                break;
            }
        }
    }

    // TODO:
    if snips_balance < snips_converter_value {
        Ok(Some(balances))
    } else {
        Ok(None)
    }
}

pub async fn transfer_excess_xpr() -> Result<Option<HashMap<String, String>>, anyhow::Error> {
    let config = load_config();
    let account_name = config.account;
    let xpr_converter_value = config.xpr_converter_value;

    let mut balances: HashMap<String, String> = get_balance()?;
    let xpr_balance_str = balances.get("XPR").unwrap_or(&"0.0000".to_string()).to_string();
    let xpr_balance = xpr_balance_str.parse::<f64>()?;

    // TODO:
    if xpr_balance > xpr_converter_value {
        let mut quantity = format!("{:.4} XPR", xpr_balance);
        let mut reduction_factor = 0.9;
        let re = Regex::new(r"overdrawn balance").unwrap();

        info!("Parsing XPR in transfer_excess_xpr: {}", quantity);

        loop {
            let output = Command::new("sh")
            .arg("-c")
            .arg(format!("proton action eosio.token transfer '{{\"from\": \"{account_name}\", \"to\": \"proton.swaps\", \"quantity\": \"{}\", \"memo\": \"SNIPSXP, 65000000\"}}' {account_name}@active", quantity))
            .output()?;

            let stdout = String::from_utf8_lossy(&output.stdout);
            info!("Transfer excess XPR output: {}", stdout);

            if re.is_match(&stdout) {
                if reduction_factor >= 0.8 {
                    quantity = format!("{:.4} XPR", xpr_balance * reduction_factor);
                    reduction_factor -= 0.1;
                } else {
                    eprintln!("Failed to transfer excess XPR after multiple reductions. Output: {}", stdout);
                    break;
                }
            } else {
                // После успешного перевода токенов, получаем обновленные балансы
                balances = get_balance()?;
                break;
            }
        }
    }

    // TODO:
    if xpr_balance < xpr_converter_value {
        Ok(Some(balances))
    } else {
        Ok(None)
    }
}