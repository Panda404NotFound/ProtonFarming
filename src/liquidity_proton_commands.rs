use anyhow::Result;
use tokio::process::Command;
use log::info;
use crate::load_config;

pub async fn prepare_deposit(symbols: &[(&str, &str)]) -> Result<(String, String)> {
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
        .arg(format!(
            "proton action proton.swaps depositprep '{{\"owner\": \"{account_name}\", \"symbols\": [{}]}}' {account_name}",
            symbols_arg
        ))
        .output()
        .await?;
 
    if output.status.success() {
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