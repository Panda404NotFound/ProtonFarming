use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub account: String,
    pub cycle_time: f64,
    pub xpr_converter_value: f64,
    pub snips_converter_value: f64,
    pub xusdc_converter_value: f64,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            account: env::var("ACCOUNT_NAME").expect("ACCOUNT_NAME не установлен в .env"),
            cycle_time: env::var("CYCLE_TIME").expect("CYCLE_TIME не установлен в .env").parse::<f64>().expect("REWARDS_CYCLE_TIME не может быть преобразован в f64"),
            xpr_converter_value: env::var("XPR_CONVERTER_VALUE").expect("XPR_CONVERTER_VALUE не установлен в .env").parse::<f64>().expect("XPR_CONVERTER_VALUE не может быть преобразован в f64"),
            snips_converter_value: env::var("SNIPS_CONVERTER_VALUE").expect("SNIPS_CONVERTER_VALUE не установлен в .env").parse::<f64>().expect("SNIPS_CONVERTER_VALUE не может быть преобразован в f64"),
            xusdc_converter_value: env::var("XUSDC_CONVERTER_VALUE").expect("XUSDC_CONVERTER_VALUE не установлен в .env").parse::<f64>().expect("XUSDC_CONVERTER_VALUE не может быть преобразован в f64"),
        }
    }
}

pub fn load_config() -> Config {
    // Здесь вы можете добавить логику для загрузки конфигурации из файла,
    // если это необходимо. Пока что мы просто возвращаем значения по умолчанию.
    Config::default()
}