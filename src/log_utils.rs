use log4rs::{
    append::file::FileAppender,
    config::{Appender, Root},
    encode::pattern::PatternEncoder,
    Config,
};

use crate::{app_constants::AppConstants, windows_os_utils::build_absolute_path};

pub fn start_logging() {
    // set up logging

    let logfile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new(
            "{d(%Y-%m-%d %H:%M:%S.%3f)(local)} {l} {t} - {m}{n}",
        )))
        .build(AppConstants::LOG_FILE_LOCATION.to_string() + AppConstants::APP_NAME + ".log")
        .unwrap();

    let log_config = Config::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .build(
            Root::builder()
                .appender("logfile")
                .build(log::LevelFilter::Info),
        );

    let _config_handle = log4rs::init_config(log_config.unwrap());
}

pub fn open_logs_externally() {
    match std::process::Command::new("notepad")
        .arg(build_absolute_path(
            &(AppConstants::LOG_FILE_LOCATION.to_string() + AppConstants::APP_NAME + ".log"),
        ))
        .output()
    {
        Ok(_) => {}
        Err(_) => {
            log::error!("Unable to open notepad to access application logs")
        }
    };
}
