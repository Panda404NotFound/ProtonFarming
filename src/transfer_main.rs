use anyhow::Result;
use log::info;
use crate::load_config;

use crate::transfer_balance::get_balance;
use crate::transfer_proton_commands::{prepare_deposit_wrapper, transfer_tokens, transfer_excess_snips, transfer_excess_xpr};

pub async fn transfer_main() -> Result<()> {
    let config = load_config();
    let snips_converter_value = config.snips_converter_value;

    // Получаем баланс токенов
    let mut balances = get_balance()?;
    info!("Token balances: {:?}", balances);

    // Выполняем transfer_excess_snips
    if let Some(updated_balances) = transfer_excess_snips().await? {
        balances = updated_balances;
    }

    // Выполняем transfer_excess_xpr
    if let Some(updated_balances) = transfer_excess_xpr().await? {
        balances = updated_balances;
    }

    // Получаем значения балансов для SNIPS, XPR и XUSDC
    let xpr_balance = balances.get("XPR").unwrap_or(&"0.0000".to_string()).parse::<f64>()?;
    let snips_balance = balances.get("SNIPS").unwrap_or(&"0.0000".to_string()).parse::<f64>()?;
    let xusdc_balance = balances.get("XUSDC").unwrap_or(&"0.000000".to_string()).parse::<f64>()?;

    // Определяем значение для передачи 0.2222 SNIPS и сохраняем остальные значения в переменных
    let fixed_snips_value = format!("{:.4} SNIPS", 0.2222);
    let snips_value = format!("{:.4} SNIPS", snips_balance);
    let xpr_transfer_value = format!("{:.4} XPR", xpr_balance);
    let xusdc_transfer_value = format!("{:.6} XUSDC", xusdc_balance);

    // Определяем значение для передачи SNIPS в зависимости от условия
    // TODO:
    let snips_transfer_value = if snips_balance < snips_converter_value {
        fixed_snips_value.clone()
    } else {
        snips_value.clone()
    };
    
    match prepare_deposit_wrapper(&[("4,XPR", "eosio.token")]).await {
        Ok(_) => info!("Prepare deposit wrapper for XPR completed successfully"),
        Err(e) => eprintln!("Error in prepare deposit wrapper for XPR: {}", e)
    }
    match prepare_deposit_wrapper(&[("4,SNIPS", "snipcoins")]).await {
        Ok(_) => info!("Prepare deposit wrapper for SNIPS completed successfully"),
        Err(e) => eprintln!("Error in prepare deposit wrapper for SNIPS: {}", e)
    }
    
    match prepare_deposit_wrapper(&[("6,XUSDC", "xtokens")]).await {
        Ok(_) => info!("Prepare deposit wrapper for XUSDC completed successfully"),
        Err(e) => eprintln!("Error in prepare deposit wrapper for XUSDC: {}", e)
    }

    // Запускаем синхронную последовательную обработку transfer только для токенов с балансом выше 0
    match snips_balance {
        x if x > 0.0 => {
            match transfer_tokens("snipcoins", &snips_transfer_value, "proton.swaps", "deposit").await {
                Ok(_) => info!("SNIPS transfer completed successfully"),
                Err(e) => eprintln!("Error in SNIPS transfer task: {}", e)
            }
        },
        _ => info!("Skipping SNIPS transfer due to zero balance")
    }

    match xpr_balance {
        x if x > 0.0 => {
            match transfer_tokens("eosio.token", &xpr_transfer_value, "proton.swaps", "deposit").await {
                Ok(_) => info!("XPR transfer completed successfully"), 
                Err(e) => eprintln!("Error in XPR transfer task: {}", e)
            }
        },
        _ => info!("Skipping XPR transfer due to zero balance")
    }

    match xusdc_balance {
        x if x > 0.0 => {
            match transfer_tokens("xtokens", &xusdc_transfer_value, "proton.swaps", "deposit").await {
                Ok(_) => info!("XUSDC transfer completed successfully"),
                Err(e) => eprintln!("Error in XUSDC transfer task: {}", e)
            }
        },
        _ => info!("Skipping XUSDC transfer due to zero balance")
    }
    Ok(())
}