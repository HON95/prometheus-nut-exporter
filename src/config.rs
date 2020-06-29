use std::env;
use std::fs;

#[derive(Debug, Clone)]
pub struct Config {
    pub version: String,
    pub http_port: u16,
    pub http_path: String,
    pub log_requests_console: bool,
}

impl Config {
    const DEFAULT_VERSION: &'static str = "0.0.0";
    const DEFAULT_HTTP_PORT: u16 = 9995;
    const DEFAULT_HTTP_PATH: &'static str = "/nut";
    const DEFAULT_LOG_REQUESTS_CONSOLE: bool = false;
}

const VERSION_FILE: &str = "VERSION";

pub fn read_config() -> Config {
    let mut config = Config {
        version: Config::DEFAULT_VERSION.to_owned(),
        http_port: Config::DEFAULT_HTTP_PORT,
        http_path: Config::DEFAULT_HTTP_PATH.to_owned(),
        log_requests_console: Config::DEFAULT_LOG_REQUESTS_CONSOLE,
    };

    read_version_file(&mut config);

    if let Ok(http_port_str) = env::var("HTTP_PORT") {
        if let Ok(http_port) = http_port_str.parse::<u16>() {
            config.http_port = http_port;
        }
    }
    if let Ok(http_path) = env::var("HTTP_PATH") {
        if http_path.starts_with('/') {
            config.http_path = http_path;
        }
    }
    if let Ok(log_requests_console_str) = env::var("LOG_REQUESTS_CONSOLE") {
        if let Ok(log_requests_console) = log_requests_console_str.parse::<bool>() {
            config.log_requests_console = log_requests_console;
        }
    }

    config
}

pub fn read_version_file(config: &mut Config) {
    let content_res = fs::read_to_string(VERSION_FILE);
    match content_res {
        Ok(content) => {
            config.version = content.trim().to_owned();
            println!("Version: {}", config.version);
        },
        Err(error) => {
            eprintln!("Failed to read version file: {}", error);
        },
    };
}
