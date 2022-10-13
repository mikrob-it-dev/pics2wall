#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use crate::utils::{fetch_pic, set_pic_as_wallpaper, start_logging};
use app_constants::AppConstants;
use app_gui::MyApp;
use egui::Vec2;
use std::{fs::create_dir, io::ErrorKind, thread, time};

mod app_config;
mod app_constants;
mod app_gui;
mod utils;

fn main() {
    start_logging();
    log::info!("----------- App started -----------");
    // env::set_var("RUST_BACKTRACE", "1");

    let my_app = MyApp::default();
    let mut secs_since_last_update = 0;

    match create_dir("data") {
        Ok(_) => {}
        Err(e) => match e.kind() {
            ErrorKind::AlreadyExists => {
                log::info!("Already existing /data folder used for storing fetched images");
            }
            _ => {
                log::error!("Could not create /data file");
            }
        },
    };

    let mut thread_my_app_clone = my_app.clone();

    thread::spawn(move || loop {
        log::info!("Timer tick");

        if *thread_my_app_clone
            .config
            .is_auto_update_active
            .lock()
            .unwrap()
        {
            log::info!("Auto-update cycle started");
            if secs_since_last_update
                >= *thread_my_app_clone
                    .config
                    .auto_update_interval
                    .lock()
                    .unwrap()
                || secs_since_last_update == 0
            {
                fetch_pic(&mut thread_my_app_clone);
                set_pic_as_wallpaper(&thread_my_app_clone);
                secs_since_last_update = 0;
            }

            secs_since_last_update += AppConstants::TICK_INTERVAL;

            *thread_my_app_clone.status.lock().unwrap() = String::from("Idle");
        } else {
            secs_since_last_update = 0;
        }

        if *thread_my_app_clone
            .is_test_image_fetch_requested
            .lock()
            .unwrap()
        {
            fetch_pic(&mut thread_my_app_clone);
            *thread_my_app_clone
                .is_test_image_fetch_requested
                .lock()
                .unwrap() = false;
        }

        thread::sleep(time::Duration::from_secs(AppConstants::TICK_INTERVAL));
    });

    let icon = image::open("resources/icon.png")
        .expect("Failed to open icon path")
        .to_rgba8();
    let (icon_width, icon_height) = icon.dimensions();

    // let options = eframe::NativeOptions::default();
    let options = eframe::NativeOptions {
        initial_window_size: Option::from(Vec2::new(500.0, 500.0)),
        icon_data: Some(eframe::IconData {
            rgba: icon.into_raw(),
            width: icon_width,
            height: icon_height,
        }),
        ..Default::default()
    };

    eframe::run_native(
        AppConstants::APP_NAME,
        options,
        Box::new(|_cc| Box::new(my_app)),
    );
}
