use std::{
    fs::File,
    io::{Read, Write},
    sync::{Arc, Mutex},
};

use serde::{Deserialize, Serialize};

use crate::{app_constants::AppConstants, app_gui::Enum};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct AppConfig {
    pub image_address: Arc<Mutex<String>>,
    pub auto_update_interval: Arc<Mutex<u64>>,
    pub is_auto_update_active: Arc<Mutex<bool>>,
    pub wallpaper_fit_style: Arc<Mutex<Enum>>,
    pub file_type: String,
}

impl AppConfig {
    pub fn default() -> Self {
        Self {
            image_address: Arc::new(Mutex::new(String::from(
                "https://www.mikrob.it/blank_online_test_page.jpg",
            ))),
            auto_update_interval: Arc::new(Mutex::new(AppConstants::AUTO_UPDATE_MIN_INTERVAL)),
            is_auto_update_active: Arc::new(Mutex::new(false)),
            wallpaper_fit_style: Arc::new(Mutex::new(Enum::Center)),
            file_type: String::from(""),
        }
    }

    pub fn save_app_config(&self) -> String {
        let serialized_config_json = serde_json::to_string_pretty(&self).unwrap();

        let config_file = File::create(AppConstants::CONFIG_FILE_LOCATION);
        match config_file {
            Ok(_) => {
                let file_write_result = config_file
                    .unwrap()
                    .write_all(serialized_config_json.as_bytes());
                match file_write_result {
                    Ok(_) => {}
                    Err(_) => log::error!("Failed writing the config file"),
                }
            }
            Err(_) => log::error!("Failed creating the config file"),
        }

        serialized_config_json
    }

    pub fn load_app_config() -> Self {
        let serialized_config_in_file = File::open(AppConstants::CONFIG_FILE_LOCATION);

        match &serialized_config_in_file {
            Ok(_) => {}
            Err(_) => {
                log::error!("Config file load failed, loading default configuration");
                return AppConfig::default();
            }
        }

        let mut buffer = String::new();
        let config_file_read_result = serialized_config_in_file
            .unwrap()
            .read_to_string(&mut buffer);

        match config_file_read_result {
            Ok(_) => {
                log::info!("Config file read successfully: {}", &buffer)
            }
            Err(_) => log::error!("Config file read failed"),
        }

        let config_deserialize_result: Result<AppConfig, serde_json::Error> =
            serde_json::from_str(&buffer);

        match config_deserialize_result {
            Ok(_) => {
                // TODO: Super ugly, rewrite

                if *config_deserialize_result
                    .as_ref()
                    .unwrap()
                    .auto_update_interval
                    .lock()
                    .unwrap()
                    < AppConstants::AUTO_UPDATE_MIN_INTERVAL
                {
                    log::error!("Invalid configuration, the auto-update interval is too short. Loading default configuration");
                    return AppConfig::default();
                } else {
                    log::info!("Config deserialized successfully and will be used");
                    return config_deserialize_result.unwrap();
                }
            }
            Err(_) => {
                log::error!("Config file deserialize failed, loading default configuration");
                AppConfig::default()
            }
        }
    }
}
