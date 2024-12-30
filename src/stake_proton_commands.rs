use std::process::Command;
use anyhow::Result;
use regex::Regex;
use crate::stake_balance::get_balance;
use log::info;
use crate::load_config;

pub async fn open_stake(stakes: &[&str]) -> Result<()> {
    let config = load_config();
    let account_name = config.account;

    info!("p_c.rs: Start open_stake...");
    let stakes_arg = stakes.iter().map(|s| format!("\"{}\"", s)).collect::<Vec<_>>().join(", ");
    
    let output = Command::new("sh")
    .arg("-c")
    .arg(format!("proton action yield.farms open '{{\"user\": \"{account_name}\", \"stakes\": [{}]}}' {account_name}@active", stakes_arg))
    .output()?;
 
        if !output.status.success() {
            eprintln!("Failed to open stake. Error: {}", String::from_utf8_lossy(&output.stderr));
            return Err(anyhow::anyhow!("Failed to open stake"));
        } else {
            info!("Open stake successful");
        }
    
    Ok(())
}

pub async fn transfer_tokens_staking(contract: &str, quantity: &str, to: &str, memo: &str) -> Result<()> {
    let config = load_config();
    let account_name = config.account;

    info!("Start transfer_tokens to Stake LP...");
    
    let output = Command::new("sh")
    .arg("-c")
    .arg(format!("proton action {} transfer '{{\"from\":\"{account_name}\",\"to\":\"{}\",\"quantity\":\"{}\",\"memo\":\"{}\"}}' {account_name}@active", contract, to, quantity, memo))
    .output()?;

    if !output.status.success() {
        eprintln!("Failed to transfer tokens. Error: {}", String::from_utf8_lossy(&output.stderr));
        return Err(anyhow::anyhow!("Failed to transfer tokens"));
    } else {
        info!("Token transfer successful to Stake LP. Output: {}", String::from_utf8_lossy(&output.stdout));
    }

    Ok(())
}

// Асинхронная функция для обработки стейкинга SNIPSXP
pub async fn handle_snipsxp_staking(formatted_snipsxp: String) -> Result<()> {
    let snipsxp_staking_result = transfer_tokens_staking(
        "proton.swaps",
        &formatted_snipsxp,
        "yield.farms",
        "Stake LP"
    ).await;

    match snipsxp_staking_result {
        Ok(_) => info!("SNIPSXP liquidity sent to staking successfully"),
        Err(e) => {
            let error_message = format!("{}", e);
            let re_overdrawn_balance = Regex::new(r"assertion failure with message: overdrawn balance")?;

            if re_overdrawn_balance.is_match(&error_message) {
                let max_retries = 3;
                let mut retry_count = 0;

                while retry_count < max_retries {
                    // Обновляем баланс для SNIPSXP
                    let balances = get_balance()?;
                    let updated_snipsxp_balance = balances.get("SNIPSXP").unwrap_or(&"0.00000000".to_string()).parse::<f64>()?;

                    // Форматируем обновленный баланс для SNIPSXP
                    let updated_formatted_snipsxp = format!("{:.8} SNIPSXP", updated_snipsxp_balance);

                    // Повторяем попытку отправки ликвидности в стейкинг для SNIPSXP
                    let retry_snipsxp_staking_result = transfer_tokens_staking(
                        "proton.swaps",
                        &updated_formatted_snipsxp,
                        "yield.farms",
                        "Stake LP"
                    ).await;

                    match retry_snipsxp_staking_result {
                        Ok(_) => {
                            info!("SNIPSXP liquidity sent to staking successfully after retry");
                            break;
                        }
                        Err(e) => {
                            eprintln!("Error sending SNIPSXP liquidity to staking after retry: {}", e);
                            retry_count += 1;
                        }
                    }
                }

                if retry_count == max_retries {
                    eprintln!("Failed to send SNIPSXP liquidity to staking after {} retries", max_retries);
                }
            } else {
                eprintln!("Error sending SNIPSXP liquidity to staking: {}", e);
            }
        }
    }

    Ok(())
}

// Асинхронная функция для обработки стейкинга XPRUSDC
pub async fn handle_xprusdc_staking(formatted_xprusdc: String) -> Result<()> {
    let xprusdc_staking_result = transfer_tokens_staking(
        "proton.swaps",
        &formatted_xprusdc,
        "yield.farms",
        "Stake LP"
    ).await;

    match xprusdc_staking_result {
        Ok(_) => info!("XPRUSDC liquidity sent to staking successfully"),
        Err(e) => {
            let error_message = format!("{}", e);
            let re_overdrawn_balance = Regex::new(r"assertion failure with message: overdrawn balance")?;

            if re_overdrawn_balance.is_match(&error_message) {
                let max_retries = 3;
                let mut retry_count = 0;

                while retry_count < max_retries {
                    // Обновляем баланс для XPRUSDC
                    let balances = get_balance()?;
                    let updated_xprusdc_balance = balances.get("XPRUSDC").unwrap_or(&"0.00000000".to_string()).parse::<f64>()?;

                    // Форматируем обновленный баланс для XPRUSDC
                    let updated_formatted_xprusdc = format!("{:.8} XPRUSDC", updated_xprusdc_balance);

                    // Повторяем попытку отправки ликвидности в стейкинг для XPRUSDC
                    let retry_xprusdc_staking_result = transfer_tokens_staking(
                        "proton.swaps",
                        &updated_formatted_xprusdc,
                        "yield.farms",
                        "Stake LP"
                    ).await;

                    match retry_xprusdc_staking_result {
                        Ok(_) => {
                            info!("XPRUSDC liquidity sent to staking successfully after retry");
                            break;
                        }
                        Err(e) => {
                            eprintln!("Error sending XPRUSDC liquidity to staking after retry: {}", e);
                            retry_count += 1;
                        }
                    }
                }

                if retry_count == max_retries {
                    eprintln!("Failed to send XPRUSDC liquidity to staking after {} retries", max_retries);
                }
            } else {
                eprintln!("Error sending XPRUSDC liquidity to staking: {}", e);
            }
        }
    }

    Ok(())
}