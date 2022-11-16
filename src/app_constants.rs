#[derive(Clone)]
pub struct AppConstants {}

impl AppConstants {
    // app info
    pub const APP_NAME: &str = "pics2wall";
    pub const APP_VERSION: &str = "v0.1 (experimental)";
    pub const APP_DEVELOPER: &str = "mikrob";
    pub const APP_DEVELOPER_WEBSITE: &str = "http://mikrob.it";

    // app internal
    pub const TICK_INTERVAL: u64 = 5;
    pub const AUTO_UPDATE_MIN_INTERVAL: u64 = 60;

    // file locations
    pub const WALLPAPER_IMAGE_FILE_LOCATION: &str = r#"data/"#;
    pub const BLANK_TEST_IMAGE_FILE_LOCATION: &str =
        r#"resources\unknown_fetch_status_test_page.jpg"#;
    pub const FAILED_TEST_IMAGE_FILE_LOCATION: &str =
        r#"resources\failed_fetch_status_test_page.jpg"#;
    pub const IN_PROGRESS_TEST_IMAGE_FILE_LOCATION: &str =
        r#"resources\in_progress_fetch_status_test_page.jpg"#;
    pub const CONFIG_FILE_LOCATION: &str = "config.json";
    pub const LOG_FILE_LOCATION: &str = "log/";

    // TODO: format better
    pub const LICENSE_TEXT: &str =

"Copyright © 2022 mikrob\n\n
Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the \"Software\"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:\n\n
The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.\n\n
THE SOFTWARE IS PROVIDED \"AS IS\", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.";
}
