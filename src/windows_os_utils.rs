use std::{ffi::OsStr, iter, os::windows::prelude::OsStrExt, path::PathBuf, env::current_dir};

use winapi::{um::winuser::{SystemParametersInfoW, SPI_SETDESKWALLPAPER, SPIF_SENDCHANGE, SPIF_UPDATEINIFILE, SetSysColors, COLOR_BACKGROUND}, ctypes::c_void};
use winreg::{enums::HKEY_CURRENT_USER, RegKey};

use crate::{app_gui::{MyApp, Enum}, app_constants::AppConstants, common_utils::color32_to_reversed_u32};

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

pub fn build_absolute_path(relative_path_str: &str) -> PathBuf {
    current_dir().unwrap().join(relative_path_str)
}