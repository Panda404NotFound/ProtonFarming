// src/main.rs

pub mod farming_main;
pub mod liquidity_liquidity;
pub mod liquidity_main;
pub mod liquidity_pools;
pub mod liquidity_proton_commands;
pub mod stake_main;
pub mod stake_balance;
pub mod stake_proton_commands;
pub mod transfer_balance;
pub mod transfer_main;
pub mod transfer_proton_commands;
pub mod withdrawall_main;
pub mod config;

use dotenv::dotenv;
use env_logger;
use log::info;
use std::time::{Duration, Instant};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::config::load_config;
use crate::liquidity_main::liquidity_main;
use crate::farming_main::farming_main;
use crate::stake_main::stake_main;
use crate::transfer_main::transfer_main;
use crate::withdrawall_main::withdrawall_main;

#[tokio::main]
async fn main() {

    // Инициализация логгера и загрузка переменных окружения из .env файла
    env_logger::init();
    dotenv().ok();
    let config = load_config();

    // Инициализация переменных для цикла
    let cycle_duration = Duration::from_secs_f64(config.cycle_time);
    let last_run = Arc::new(AtomicBool::new(false));
    let mut last_execution = Instant::now();
    
    // Цикл операций с счетчиком времени
    loop {
        let now = Instant::now();
        if now.duration_since(last_execution) >= cycle_duration {
            if !last_run.load(Ordering::SeqCst) {
                last_run.store(true, Ordering::SeqCst);
                
                // Выполнение цикла операций
                farming_main().await;

                // Затем переводим деньги в пулы
                match transfer_main().await {
                    Ok(_) => {
                        info!("Transfer completed successfully");
                    }
                    Err(e) => {
                        eprintln!("Error in transfer main: {}", e);
                    }
                }

                // Затем добавляем ликвидность в пулы
                match liquidity_main().await {
                    Ok(_) => {
                        info!("Liquidity added successfully");
                    }
                    Err(e) => {
                        eprintln!("Error in liquidity main: {}", e);
                    }
                }

                // Затем заводим деньги в стейкинг
                match stake_main().await {
                    Ok(_) => {
                        info!("Staking completed successfully");
                    }
                    Err(e) => {
                        eprintln!("Error in stake main: {}", e);
                    }
                }

                // Вконце выводим все средства 
                withdrawall_main().await;

                info!("Цикл завершен");
                
                last_execution = now;
                last_run.store(false, Ordering::SeqCst);
            }
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}