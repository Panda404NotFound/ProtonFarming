use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command as AsyncCommand;
use log::info;
use crate::load_config;

async fn claim_rewards(stake: &str, pool: &str) -> Result<(), anyhow::Error> {
    let config = load_config();
    let account_name = config.account;
    let command = format!(
        r#"proton action yield.farms claim '{{"claimer": "{account_name}", "stakes": ["{stake}"]}}' {account_name}"#,
    );

    let output = AsyncCommand::new("bash")
        .arg("-c")
        .arg(&command)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to execute command");

    let stdout = output.stdout.expect("Failed to get stdout");
    let stderr = output.stderr.expect("Failed to get stderr");

    let mut stdout_reader = BufReader::new(stdout).lines();
    let mut stderr_reader = BufReader::new(stderr).lines();

    while let Some(line) = stdout_reader.next_line().await.unwrap_or(None) {
        info!("Claimed rewards for stake: {} in pool: {}. Output: {}", stake, pool, line);
    }

    while let Some(line) = stderr_reader.next_line().await.unwrap_or(None) {
        eprintln!("Error claiming rewards for stake: {} in pool: {}. Error: {}", stake, pool, line);
    }

    Ok(())
}

pub async fn farming_main() {
    match claim_rewards("XPRUSDC", "XPR").await {
        Ok(_) => {
            info!("XPR rewards claimed successfully");
        }
        Err(e) => {
            eprintln!("Error in XPR rewards claim: {}", e);
        }
    }

    match claim_rewards("SNIPSXP", "SNIPS").await {
        Ok(_) => {
            info!("SNIPS rewards claimed successfully");
        }
        Err(e) => {
            eprintln!("Error in SNIPS rewards claim: {}", e);
        }
    }
}