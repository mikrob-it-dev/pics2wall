use crate::{
    app_config::AppConfig,
    app_constants::AppConstants,
    utils::{
        build_absolute_path, clear_background, get_current_background_color, is_image_url_valid,
        load_image_from_path, set_background_color,
    },
};
use egui::{Align2, Context, Label, Ui, Vec2};
use egui_extras::RetainedImage;
use serde::{Deserialize, Serialize};
use std::{
    path::Path,
    sync::{Arc, Mutex},
};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum Enum {
    Center,
    Fit,
    Stretch,
}

#[derive(Clone)]
pub struct MyApp {
    pub config: AppConfig,
    pub status: Arc<Mutex<String>>,
    pub last_fetch_result: Arc<Mutex<String>>,
    pub last_fetch_time: Arc<Mutex<String>>,
    pub test_image: Arc<Mutex<RetainedImage>>,
    pub autoupdate_interval_input_string: String,
    pub is_form_valid: bool,
    pub is_diagnostic_image_shown: bool,
    pub is_license_info_shown: bool,
    pub is_test_image_fetch_requested: Arc<Mutex<bool>>,
}

impl MyApp {
    pub fn default() -> Self {
        let loaded_config = AppConfig::load_app_config();
        let loaded_auto_update_interval = loaded_config
            .auto_update_interval
            .lock()
            .unwrap()
            .to_string();

        Self {
            // config: AppConfig::new(),
            config: loaded_config,
            status: Arc::new(Mutex::new(String::from("Idle"))),
            test_image: {
                let image_path = Path::new(AppConstants::BLANK_TEST_IMAGE_FILE_LOCATION);
                Arc::new(Mutex::new(RetainedImage::from_color_image(
                    "test_image",
                    load_image_from_path(image_path).unwrap(),
                )))
            },

            // TODO: get rid of the intermediate string
            // Form-validated string for future time unit support
            autoupdate_interval_input_string: loaded_auto_update_interval,

            last_fetch_result: Arc::new(Mutex::new(String::from("N/A"))),
            last_fetch_time: Arc::new(Mutex::new(String::from("N/A"))),
            is_form_valid: true,
            is_diagnostic_image_shown: false,
            is_license_info_shown: false,
            is_test_image_fetch_requested: Arc::new(Mutex::new(false)),
        }
    }
}

// TODO: continuous refresh

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // // egui::Window::new(AppConstants::APP_NAME)
            //     .fixed_pos(Pos2::new(10.0, 10.0))
            //     .collapsible(false)
            //     .auto_sized()
            //     .default_width(500.0)
            //     .show(ctx, |ui| {
            ui_add_status(ui, self);
            ui_add_config(ui, self);
            ui_add_controls(ui, self);
            ui_add_diagnostic_tools(ui, self, _frame);
            ui_add_test_image(self, ctx);

            ui_add_dev_version_info(self, ctx);
            ui_add_license_info(self, ctx);
        });

        // egui::CentralPanel::default().show(ctx, |ui| {
        //     ui.label("test");
        // });
    }
}

fn ui_add_status(ui: &mut Ui, my_app: &mut MyApp) {
    let min_col_width = 150.0;
    egui::CollapsingHeader::new("Status")
        .default_open(true)
        .show(ui, |ui| {
            egui::Grid::new("status_grid")
                .num_columns(2)
                .min_col_width(min_col_width)
                .show(ui, |ui| {
                    ui.label("Activity:");
                    ui.label(my_app.status.lock().unwrap().to_string());

                    ui.end_row();

                    ui.label("Last fetch time:");
                    ui.label(my_app.last_fetch_time.lock().unwrap().to_string());

                    ui.end_row();

                    ui.label("Last fetch result:");
                    ui.label(my_app.last_fetch_result.lock().unwrap().to_string());
                });
        });

    ui.add_space(15.0);
}

fn ui_add_config(ui: &mut Ui, my_app: &mut MyApp) {
    let min_col_width = 150.0;
    let desired_right_col_width = 300.0;
    let is_configurable = !*my_app.config.is_auto_update_active.lock().unwrap();

    egui::CollapsingHeader::new("Configuration")
        .default_open(true)
        .enabled(is_configurable)
        .show(ui, |ui| {
            egui::Grid::new("config_grid")
                .num_columns(2)
                .min_col_width(min_col_width)
                .show(ui, |ui| {
                    ui.label("Image address:");

                    let image_address_handle = &mut *my_app.config.image_address.lock().unwrap();
                    let image_address_handle_clone = image_address_handle.clone();

                    let source_address_edit_text = egui::TextEdit::singleline(image_address_handle)
                        .desired_width(desired_right_col_width);

                    ui.add(source_address_edit_text);

                    ui.end_row();

                    ui.label("Update Interval [s]:");

                    let update_interval_edit_text =
                        egui::TextEdit::singleline(&mut my_app.autoupdate_interval_input_string)
                            .desired_width(desired_right_col_width);

                    ui.add(update_interval_edit_text);

                    let parsed_input = &mut my_app.autoupdate_interval_input_string.parse::<u64>();
                    let parsed_input_clone = parsed_input.clone();

                    // TODO: Fix ugly

                    match parsed_input {
                        Ok(_) => {
                            let mod_parsed_input = parsed_input_clone.unwrap()
                                % AppConstants::AUTO_UPDATE_MIN_INTERVAL;
                            let u64_input_string_value = parsed_input.clone().unwrap();

                            // TODO pattern match
                            if mod_parsed_input == 0
                                && u64_input_string_value != 0
                                && is_image_url_valid(&image_address_handle_clone)
                            {
                                *my_app.config.auto_update_interval.lock().unwrap() =
                                    u64_input_string_value;
                                my_app.is_form_valid = true;
                            } else {
                                my_app.is_form_valid = false;
                            }
                        }
                        _ => {
                            my_app.is_form_valid = false;
                        }
                    }

                    ui.end_row();

                    ui.label("Background color:");

                    let mut current_background_color = get_current_background_color();

                    let color_edit_button_handle =
                        ui.color_edit_button_srgb(&mut current_background_color);

                    // TODO: potentially add a delay to prevent super fast background color swap

                    if color_edit_button_handle.changed() {
                        set_background_color(current_background_color)
                    };

                    ui.end_row();

                    ui.label("Fit style:");

                    let current_wallpaper_fit_value =
                        &mut *my_app.config.wallpaper_fit_style.lock().unwrap();

                    egui::ComboBox::from_id_source("Take your pick")
                        .selected_text(format!("{:?}", current_wallpaper_fit_value))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                current_wallpaper_fit_value,
                                Enum::Center,
                                "Center",
                            );
                            ui.selectable_value(current_wallpaper_fit_value, Enum::Fit, "Fit");
                            ui.selectable_value(
                                current_wallpaper_fit_value,
                                Enum::Stretch,
                                "Stretch",
                            );
                        });

                    ui.end_row();
                });

            ui.horizontal(|ui| {
                let save_config_button_handle = ui.add_enabled(
                    my_app.is_form_valid,
                    egui::Button::new("Save configuration"),
                );

                if save_config_button_handle.clicked() {
                    my_app.config.save_app_config();
                }

                let reset_button_handle = ui.add(egui::Button::new("Reset configuration"));

                if reset_button_handle.clicked() {
                    my_app.config = AppConfig::default();
                    my_app.autoupdate_interval_input_string = my_app
                        .config
                        .auto_update_interval
                        .lock()
                        .unwrap()
                        .to_string();
                }
            });
        });

    ui.add_space(15.0);
}

fn ui_add_controls(ui: &mut Ui, my_app: &mut MyApp) {
    egui::CollapsingHeader::new("Controls")
        .default_open(true)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                let button_handle = ui.add_enabled(
                    my_app.is_form_valid,
                    egui::Button::new(if *my_app.config.is_auto_update_active.lock().unwrap() {
                        "Stop auto-update"
                    } else {
                        "Start auto-update"
                    }),
                );

                if button_handle.clicked() {
                    let sta = *my_app.config.is_auto_update_active.lock().unwrap();
                    *my_app.config.is_auto_update_active.lock().unwrap() = !sta;
                    if *my_app.config.is_auto_update_active.lock().unwrap() {my_app.is_diagnostic_image_shown = true};
                }

                let button_handle = ui.add(egui::Button::new("Clear wallpaper"));

                if button_handle.clicked() {
                    clear_background();
                }
            });

            if !my_app.is_form_valid {
                let validation_message = format!("Auto-update cannot be started, the configuration is incorrect.\n1. The auto-update interval must be divisible by {} seconds\n2. The image address must be a valid http or https URL directly pointing to an image\n3. Supported image formats / suffixes: .jpg, .jpeg, .png", AppConstants::AUTO_UPDATE_MIN_INTERVAL);
                ui.label(validation_message);
            }
        });

    ui.add_space(15.0);

}

fn ui_add_diagnostic_tools(ui: &mut Ui, my_app: &mut MyApp, _frame: &mut eframe::Frame) {
    egui::CollapsingHeader::new("Diagnostic tools")
        .default_open(false)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                let button = egui::Button::new("Test connection");
                let button_handle = ui.add_enabled(
                    my_app.status.lock().unwrap().eq("Idle")
                        && my_app.is_form_valid
                        && !*my_app.config.is_auto_update_active.lock().unwrap(),
                    button,
                );

                if button_handle.clicked() {
                    log::info!("Diagnostic image fetch attempted");
                    *my_app.is_test_image_fetch_requested.lock().unwrap() = true;
                    my_app.is_diagnostic_image_shown = true;
                }

                let test_image_button_handle = ui.button("Show test image");
                if test_image_button_handle.clicked() {
                    my_app.is_diagnostic_image_shown = true;
                }

                let button = egui::Button::new("Application logs");
                let button_handle = ui.add(button);

                if button_handle.clicked() {
                    match std::process::Command::new("notepad")
                        .arg(build_absolute_path(AppConstants::LOG_FILE_LOCATION))
                        .output()
                    {
                        Ok(_) => {}
                        Err(_) => {
                            log::error!("Unable to open notepad to access application logs")
                        }
                    };
                }
            });
        });

    ui.add_space(15.0);
}

fn ui_add_dev_version_info(my_app: &mut MyApp, ctx: &Context) {
    egui::Area::new("dev, version info")
        .anchor(Align2::RIGHT_BOTTOM, [-10.0, -10.0])
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("developed by:");
                ui.hyperlink_to(
                    format!("{}", AppConstants::APP_DEVELOPER),
                    AppConstants::APP_DEVELOPER_WEBSITE,
                );
                ui.label(format!("|  version: {}   |", AppConstants::APP_VERSION));
                let license_button_handle = ui.button("License: MIT");
                if license_button_handle.clicked() {
                    my_app.is_license_info_shown = !my_app.is_license_info_shown;
                }
            });
        });
}

fn ui_add_license_info(my_app: &mut MyApp, ctx: &Context) {
    egui::Window::new("License")
        .collapsible(false)
        .open(&mut my_app.is_license_info_shown)
        .anchor(Align2::LEFT_TOP, [10.0, 10.0])
        .min_width(470.0)
        .vscroll(true)
        .resizable(false)
        .show(ctx, |ui| {
            ui.add(Label::new(AppConstants::LICENSE_TEXT).wrap(true));
        });
}

fn ui_add_test_image(my_app: &mut MyApp, ctx: &Context) {
    egui::Window::new("Test image")
        .collapsible(false)
        .open(&mut my_app.is_diagnostic_image_shown)
        .anchor(Align2::RIGHT_TOP, [-10.0, 150.0])
        .show(ctx, |ui| {
            ui.image(
                my_app.test_image.lock().unwrap().texture_id(ctx),
                Vec2::new(250.0, 200.0),
            );
        });
}
