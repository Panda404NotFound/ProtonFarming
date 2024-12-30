use std::time::Duration;
use tokio;
use std::process::Command;
use anyhow::Result;
use regex::Regex;
use log::info;
use crate::load_config;

async fn prepare_deposit(symbols: &[(&str, &str)]) -> Result<(), anyhow::Error> {
    info!("Start prepare_deposit...");

    let config = load_config();
    let account_name = config.account; 

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
        Ok(())
    } else {
        eprintln!(
            "Failed to prepare deposit. Error: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        Err(anyhow::anyhow!("Failed to prepare deposit"))
    }
}

async fn withdraw_all() -> Result<(), anyhow::Error> {
    let config = load_config();
    let account_name = config.account;

    info!("p_c.rs: Start withdraw_all...");

    let error_regex = Regex::new(r"AbortError: The user aborted a request|Please create record at deposit table first with depositprep").unwrap();
    let mut attempts = 0;

    loop {
        let output = Command::new("sh")
        .arg("-c")
        .arg(format!("proton action proton.swaps withdrawall '{{\"owner\": \"{account_name}\"}}' {account_name}"))
        .output()?;

        let output_str = String::from_utf8_lossy(&output.stdout);

        if output.status.success() && !error_regex.is_match(&output_str) {
            info!("Withdraw all successful. Output: {}", output_str);
            return Ok(());
        } else {
            attempts += 1;
            eprintln!("Attempt {}: Failed to withdraw all. Error: {}", attempts, output_str);

            if attempts >= 3 {
                return Err(anyhow::anyhow!("Failed to withdraw all after 3 attempts"));
            } else {
                tokio::time::sleep(Duration::from_secs(2)).await; // Wait for 2 seconds before the next attempt
            }
        }
    }
}

pub async fn withdrawall_main() {
    match run_withdrawall_main_loop().await {
        Ok(_) => {
            info!("Main loop completed successfully.");
        }
        Err(e) => {
            eprintln!("Error in main loop: {}", e);
        }
    }
}

async fn run_withdrawall_main_loop() -> Result<(), anyhow::Error> {
    info!("Proton.swaps withdrawall");

    // Подготовка депозита для XUSDC и SNIPS
    let xusdc_symbols = &[("6,XUSDC", "xtokens"), ("4,XPR", "eosio.token")];
    let snips_symbols = &[("4,SNIPS", "snipcoins"), ("4,XPR", "eosio.token")];

    match prepare_deposit(xusdc_symbols).await {
        Ok(_) => info!("Prepare deposit for XUSDC completed successfully"),
        Err(e) => eprintln!("Error in prepare deposit for XUSDC: {}", e)
    }

    match prepare_deposit(snips_symbols).await {
        Ok(_) => info!("Prepare deposit for SNIPS completed successfully"),
        Err(e) => eprintln!("Error in prepare deposit for SNIPS: {}", e)
    }

    match withdraw_all().await {
        Ok(_) => info!("Withdraw all successful."),
        Err(e) => {
            eprintln!("Error in withdraw_all: {}", e);
            return Err(e);
        }
    }

    Ok(())
}