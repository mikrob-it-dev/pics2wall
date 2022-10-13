use std::{
    env::current_dir,
    ffi::OsStr,
    fs::File,
    io::Write,
    iter,
    os::windows::prelude::OsStrExt,
    path::{Path, PathBuf},
};

use chrono::Local;
use curl::easy::Easy;
use egui::ColorImage;
use egui_extras::RetainedImage;
use image::ImageError;
use log4rs::{
    append::file::FileAppender,
    config::{Appender, Root},
    encode::pattern::PatternEncoder,
    Config,
};
use winapi::{
    ctypes::c_void,
    um::winuser::{
        GetSysColor, SetSysColors, SystemParametersInfoW, COLOR_BACKGROUND, SPIF_SENDCHANGE,
        SPIF_UPDATEINIFILE, SPI_SETDESKWALLPAPER,
    },
};
use winreg::{enums::HKEY_CURRENT_USER, RegKey};

use crate::app_gui::MyApp;
use crate::{app_constants::AppConstants, app_gui::Enum};

pub fn color32_to_reversed_u32(color: [u8; 3]) -> u32 {
    let hex_string = format!("{:02X}{:02X}{:02X}", color[2], color[1], color[0]);
    u32::from_str_radix(&hex_string, 16).unwrap()
}

pub fn get_current_background_color() -> [u8; 3] {
    let test: u32;
    unsafe {
        test = GetSysColor(1);
        let red: u8 = (test % 256).try_into().unwrap();
        let green: u8 = (test / 256 % 256).try_into().unwrap();
        let blue: u8 = (test / 256 / 256 % 256).try_into().unwrap();
        [red, green, blue]
    }
}

pub fn load_image_from_path(path: &std::path::Path) -> Result<ColorImage, ImageError> {
    let image_file_open = image::io::Reader::open(path);

    let image = image_file_open.unwrap().decode()?;

    let size = [image.width() as _, image.height() as _];
    let image_buffer = image.to_rgba8();
    let pixels = image_buffer.as_flat_samples();
    Ok(egui::ColorImage::from_rgba_unmultiplied(
        size,
        pixels.as_slice(),
    ))
}

pub fn fetch_pic(my_app: &mut MyApp) {
    //TODO: Structure better

    let mut is_fetch_successful = false;

    log::info!(
        "Fetching image from {}",
        &my_app.config.image_address.lock().unwrap()
    );

    set_test_image_from_path(
        my_app,
        Path::new(&(AppConstants::IN_PROGRESS_TEST_IMAGE_FILE_LOCATION)),
    );

    let mut file_type = "unknown";
    let binding = &my_app.config.image_address.lock().unwrap().clone()[..];

    if binding.ends_with("png") {
        file_type = "png";
    } else if binding.ends_with("jpg") || binding.ends_with("jpeg") {
        file_type = "jpg";
    }

    *my_app.status.lock().unwrap() = String::from("Fetching");

    let mut easy = Easy::new();

    let url_set_result = easy.url(&my_app.config.image_address.lock().unwrap());
    match url_set_result {
        Ok(_) => {}
        Err(_) => {
            log::error!("Setting URL for the fetch handle failed");
        }
    }

    let redirect = easy.follow_location(true);
    match redirect {
        Ok(_) => {}
        Err(_) => log::error!("Setting redirect allowed failed"),
    }

    let mut dst = Vec::new();
    {
        let mut transfer = easy.transfer();
        transfer
            .write_function(|data| {
                dst.extend_from_slice(data);
                Ok(data.len())
            })
            .expect("Setting write function for transfer failed");

        let transfer_result = transfer.perform();

        *my_app.last_fetch_time.lock().unwrap() =
            Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        match transfer_result {
            Ok(_) => {
                is_fetch_successful = true;
                *my_app.last_fetch_result.lock().unwrap() = "Succeeded".to_string();

                AppConstants::WALLPAPER_IMAGE_FILE_LOCATION
            }
            Err(_) => {
                log::error!("Could not fetch image");
                *my_app.last_fetch_result.lock().unwrap() = "Failed".to_string();
                AppConstants::FAILED_TEST_IMAGE_FILE_LOCATION
            }
        };
    }

    if is_fetch_successful {
        let mut file = File::create(
            AppConstants::WALLPAPER_IMAGE_FILE_LOCATION.to_string() + "current." + file_type,
        )
        .unwrap();
        my_app.config.file_type = file_type.to_string();
        file.write_all(dst.as_slice()).unwrap();
    } else {
        my_app.config.file_type = "jpg".to_string();
        std::fs::copy(
            AppConstants::FAILED_TEST_IMAGE_FILE_LOCATION,
            AppConstants::WALLPAPER_IMAGE_FILE_LOCATION.to_string()
                + "current."
                + &my_app.config.file_type,
        )
        .expect("Could not copy failed test image as current image");
    }

    set_test_image_from_path(
        my_app,
        Path::new(
            &(AppConstants::WALLPAPER_IMAGE_FILE_LOCATION.to_string()
                + "current."
                + &my_app.config.file_type),
        ),
    );

    *my_app.status.lock().unwrap() = String::from("Idle");

    log::info!("Image saved");
}

pub fn set_pic_as_wallpaper(my_app: &MyApp) {
    let path = &build_absolute_path(
        &(AppConstants::WALLPAPER_IMAGE_FILE_LOCATION.to_string()
            + "current."
            + &my_app.config.file_type),
    );

    let path_os_string = OsStr::new(path)
        .encode_wide()
        .chain(iter::once(0))
        .collect::<Vec<u16>>();

    *my_app.status.lock().unwrap() = String::from("Setting");
    log::info!("Setting the wallpaper path to {:?}", path);

    unsafe {
        //TODO: Consider config of tiling
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let (desktop, _) = hkcu.create_subkey(r"Control Panel\Desktop").unwrap();
        desktop.set_value("TileWallpaper", &"0").unwrap();

        //TODO: Consider more config options (Span, )
        desktop
            .set_value(
                "WallpaperStyle",
                &match *my_app.config.wallpaper_fit_style.lock().unwrap() {
                    Enum::Center => "0",
                    Enum::Fit => "6",
                    Enum::Stretch => "2",
                },
            )
            .unwrap();

        let set_wallpaper_result = SystemParametersInfoW(
            SPI_SETDESKWALLPAPER,
            0,
            path_os_string.as_ptr() as *mut c_void,
            SPIF_UPDATEINIFILE | SPIF_SENDCHANGE,
        );

        if set_wallpaper_result == 1 {
            log::info!("Wallpaper set successfully");
        } else {
            log::error!(
                "Wallpaper setting failed - {}",
                std::io::Error::last_os_error()
            );
        }
    }
}

pub fn clear_background() {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (desktop, _) = hkcu.create_subkey(r"Control Panel\Desktop").unwrap();
    desktop.set_value("Wallpaper", &"").unwrap();

    unsafe {
        SystemParametersInfoW(
            SPI_SETDESKWALLPAPER,
            0,
            "".as_ptr() as *mut c_void,
            SPIF_UPDATEINIFILE | SPIF_SENDCHANGE,
        );
    }
}

pub fn set_background_color(color: [u8; 3]) {
    // set in registry (for persistence)
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (desktop, _) = hkcu.create_subkey(r"Control Panel\Colors").unwrap();
    let color_string =
        color[0].to_string() + " " + &color[1].to_string() + " " + &color[2].to_string();
    desktop.set_value("Background", &color_string).unwrap();

    // set through WinAPI (for immediate effect)
    unsafe {
        SetSysColors(
            1,
            &COLOR_BACKGROUND as *const i32,
            &color32_to_reversed_u32(color) as *const u32,
        );
    }
}

pub fn start_logging() {
    // set up logging

    let logfile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new(
            "{d(%Y-%m-%d %H:%M:%S.%3f)(local)} {l} {t} - {m}{n}",
        )))
        .build(AppConstants::LOG_FILE_LOCATION)
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

pub fn build_absolute_path(relative_path_str: &str) -> PathBuf {
    current_dir().unwrap().join(relative_path_str)
}

pub fn is_image_url_valid(url: &str) -> bool {
    if (url.starts_with("http") || url.starts_with("https"))
        && url.contains("://")
        && (url.ends_with(".jpg") || url.ends_with(".jpeg") || url.ends_with(".png"))
    {
        true
    } else {
        false
    }
}

pub fn set_test_image_from_path(my_app: &mut MyApp, path: &Path) {
    let img = load_image_from_path(path);

    // TODO: Fix ugly
    match img {
        Ok(_) => {
            *my_app.test_image.lock().unwrap() =
                RetainedImage::from_color_image("test_image", img.unwrap());
        }
        Err(_) => {
            my_app.config.file_type = "jpg".to_string();
            std::fs::copy(
                AppConstants::FAILED_TEST_IMAGE_FILE_LOCATION,
                AppConstants::WALLPAPER_IMAGE_FILE_LOCATION.to_string()
                    + "current."
                    + &my_app.config.file_type,
            )
            .unwrap();

            let img = load_image_from_path(Path::new(
                &(AppConstants::WALLPAPER_IMAGE_FILE_LOCATION.to_string()
                    + "current."
                    + &my_app.config.file_type),
            ));

            *my_app.test_image.lock().unwrap() =
                RetainedImage::from_color_image("test_image", img.unwrap());
        }
    }
}
