use std::time::Duration;
use tokio;
use anyhow::Result;
use std::sync::atomic::{AtomicBool, Ordering};
use log::info;

use crate::liquidity_proton_commands::prepare_deposit;
use crate::liquidity_pools;
use crate::liquidity_pools::{get_pools, update_pools_periodically};
use crate::liquidity_liquidity::{add_liquidity_wrapper_snipsxp, add_liquidity_wrapper_xprxusdc};

static POOLS_INITIALIZED: AtomicBool = AtomicBool::new(false);

pub async fn liquidity_main() -> Result<()> {
    match update_pools_periodically().await {
        Ok(_) => {
            info!("Pools updated successfully");
        }
        Err(e) => {
            eprintln!("Error updating pools: {}", e);
        }
    }

    let xpr_amount = 111111.1111;
    let xusdc_amount = 1111.111111;

    // Ждем, пока значения пулов не будут инициализированы
    while !POOLS_INITIALIZED.load(Ordering::Relaxed) {
        let (snips_pool, xpr_pool, xusdc_pool, xpr_for_xusdc_pool) = get_pools()?;
        if snips_pool > 0.0 && xpr_pool > 0.0 && xusdc_pool > 0.0 && xpr_for_xusdc_pool > 0.0 {
            POOLS_INITIALIZED.store(true, Ordering::Relaxed);
            info!("Pools initialized: SNIPS={}, XPR={}, XUSDC={}, XPRforUSDC={}", snips_pool, xpr_pool, xusdc_pool, xpr_for_xusdc_pool);
        } else {
            eprintln!("One or more pool values are zero. Waiting...");
            tokio::time::sleep(Duration::from_secs(3)).await;
        }
    }
    
    // Получаем информацию о пулах
    let (snips_pool, xpr_pool, xusdc_pool, xpr_for_xusdc_pool) = get_pools()?;
    info!("Pool information: SNIPS={}, XPR={}, XUSDC={}, XPRforUSDC={}", snips_pool, xpr_pool, xusdc_pool, xpr_for_xusdc_pool);

    // Базовые значения для добавления ликвидности
    // let mut snips_amount = 0.1618;


    // Вычисляем значение SNIPS на основе отношения токенов в пуле SNIPSXP
    let snips_amount = xpr_amount * liquidity_pools::calculate_token_ratio_snips_for_xpr(snips_pool, xpr_pool);

    // Вычисляем значение XPR на основе отношения токенов в пуле XPRUSDC
    let xpr_for_xusdc_amount = xusdc_amount * liquidity_pools::calculate_token_ratio_xpr_for_xusdc(xusdc_pool, xpr_for_xusdc_pool);

    match prepare_deposit(&[("6,XUSDC", "xtokens"), ("4,XPR", "eosio.token")]).await {
        Ok(_) => {
            info!("Main.rs: Prepare deposit successful.");
        }
        Err(e) => {
            eprintln!("Error preparing deposit: {}", e);
        }
    }

    // Добавляем ликвидность в пулы
    let snips_xpr_result = match add_liquidity_wrapper_snipsxp(snips_amount, xpr_amount).await {
        Ok(_) => {
            info!("Main.rs: Add liquidity to SNIPSXP pool successful.");
            Ok(())
        },
        Err(e) => {
            eprintln!("Error adding liquidity to SNIPSXP pool: {}", e);
            Err(e)
        }
    };

    let xpr_xusdc_result = match add_liquidity_wrapper_xprxusdc(xpr_for_xusdc_amount, xusdc_amount).await {
        Ok(_) => {
            info!("Main.rs: Add liquidity to XPRXUSDC pool successful.");
            Ok(())
        },
        Err(e) => {
            eprintln!("Error adding liquidity to XPRXUSDC pool: {}", e);
            Err(e)
        }
    };

    if let Err(e) = snips_xpr_result {
        eprintln!("Error spawning SNIPSXP liquidity task: {}", e);
    }
    if let Err(e) = xpr_xusdc_result {
        eprintln!("Error spawning XPRXUSDC liquidity task: {}", e);
    }
    Ok(())
}
