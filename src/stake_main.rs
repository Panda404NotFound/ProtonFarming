use tokio;
use anyhow::Result;
use std::time::Duration;
use log::info;

use crate::stake_proton_commands::open_stake;
use crate::stake_proton_commands::{handle_snipsxp_staking, handle_xprusdc_staking};
use crate::stake_balance::get_balance;

pub async fn stake_main() -> Result<()> {
    
    // Получаем баланс токенов
    let mut balances = get_balance()?;
    info!("Token balances: {:?}", balances);

    // Проверяем, если все значения баланса равны нулю
    let all_balances_zero = balances.values().all(|balance| balance == "0.00000000");

    if all_balances_zero {
        // Повторяем запрос баланса на 3 попытки
        let max_retries = 3;
        let mut retry_count = 0;

        while retry_count < max_retries {
            balances = get_balance()?;
            if !balances.values().all(|balance| balance == "0.00000000") {
                break;
            }
            retry_count += 1;
        }

        // Если после 3 попыток баланс все еще равен нулю, возвращаем ошибку
        if retry_count == max_retries {
            return Ok(());
        }
    }

    // Получаем значения балансов для SNIPSXP и XPRUSDC
    let xprsnips_balance = balances.get("SNIPSXP").unwrap_or(&"0.00000000".to_string()).parse::<f64>()?;
    let xprxusdc_balance = balances.get("XPRUSDC").unwrap_or(&"0.00000000".to_string()).parse::<f64>()?;

    if xprsnips_balance > 0.0 {
        info!("Opening staking for SNIPSXP");
        let _ = open_stake(&["SNIPSXP"]);
    }

    tokio::time::sleep(Duration::from_millis(1500)).await;

    if xprxusdc_balance > 0.0 {
        info!("Opening staking for XPRUSDC ");
        let _ = open_stake(&["XPRUSDC"]);
    }

    // Добавляем задержку после открытия стейкинга
    tokio::time::sleep(Duration::from_millis(1500)).await;

    // Сохраняем форматированные строки перед использованием
    let formatted_snipsxp = format!("{:.8} SNIPSXP", xprsnips_balance);
    let formatted_xprusdc = format!("{:.8} XPRUSDC", xprxusdc_balance);

    // Запускаем синхронную последовательную обработку стейкинга только для пулов с балансом выше 0
    match xprsnips_balance {
        x if x > 0.0 => {
            match handle_snipsxp_staking(formatted_snipsxp).await {
                Ok(_) => info!("SNIPSXP staking completed successfully"),
                Err(e) => eprintln!("Error in SNIPSXP staking task: {}", e)
            }
        },
        _ => info!("Skipping SNIPSXP staking due to zero balance")
    }

    match xprxusdc_balance {
        x if x > 0.0 => {
            match handle_xprusdc_staking(formatted_xprusdc).await {
                Ok(_) => info!("XPRUSDC staking completed successfully"),
                Err(e) => eprintln!("Error in XPRUSDC staking task: {}", e)
            }
        },
        _ => info!("Skipping XPRUSDC staking due to zero balance")
    }
    Ok(())
}