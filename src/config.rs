use std::net::{IpAddr, Ipv6Addr};

#[derive(Debug, Clone)]
pub struct Config {
    pub http_address: IpAddr,
    pub http_port: u16,
    pub http_path: String,
    pub print_metrics_and_exit: bool,
}

impl Config {
    const DEFAULT_HTTP_ADDRESS: IpAddr = IpAddr::V6(Ipv6Addr::UNSPECIFIED);
    const DEFAULT_HTTP_PORT: u16 = 9995;
    const DEFAULT_HTTP_PATH: &'static str = "/nut";
    const DEFAULT_PRINT_METRICS_AND_EXIT: bool = false;
}

pub fn read_config() -> Config {
    let mut config = Config {
        http_address: Config::DEFAULT_HTTP_ADDRESS,
        http_port: Config::DEFAULT_HTTP_PORT,
        http_path: Config::DEFAULT_HTTP_PATH.to_owned(),
        print_metrics_and_exit: Config::DEFAULT_PRINT_METRICS_AND_EXIT,
    };


    if let Ok(http_address_str) = std::env::var("HTTP_ADDRESS") {
        if let Ok(http_address) = http_address_str.parse::<IpAddr>() {
            config.http_address = http_address;
        }
    }

    if let Ok(http_port_str) = std::env::var("HTTP_PORT") {
        if let Ok(http_port) = http_port_str.parse::<u16>() {
            config.http_port = http_port;
        }
    }
    if let Ok(http_path) = std::env::var("HTTP_PATH") {
        if http_path.starts_with('/') {
            config.http_path = http_path;
        }
    }
    if let Ok(print_metrics_and_exit_str) = std::env::var("PRINT_METRICS_AND_EXIT") {
        if let Ok(print_metrics_and_exit) = print_metrics_and_exit_str.parse::<bool>() {
            config.print_metrics_and_exit = print_metrics_and_exit;
        }
    }

    config
}
