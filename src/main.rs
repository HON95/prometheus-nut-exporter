use std::collections::HashMap;
use std::convert::Infallible;
use std::env;
use std::io::Error;
use std::net::{IpAddr, Ipv6Addr, SocketAddr};

use chrono::Local;
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use hyper::service::{make_service_fn, service_fn};
use hyper::server::conn::AddrStream;
use lazy_static::lazy_static;
use regex::Regex;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use url::form_urlencoded;

#[derive(Debug, Clone)]
struct Config {
    http_port: u16,
    http_path: String,
    log_requests_console: bool,
}

impl Config {
    const DEFAULT_HTTP_PORT: u16 = 9999;
    const DEFAULT_HTTP_PATH: &'static str = "/nut";
    const DEFAULT_LOG_REQUESTS_CONSOLE: bool = false;
}

#[derive(Debug, Clone)]
struct UpsVar {
    ups: String,
    variable: String,
    value: String,
}

const UPS_DESCRIPTION_PSEUDOVAR: &'static str = "description";

/*
 * TODO
 * OpenMetrics /metrics endpoint
 * Check if unabailable, tmp and perm. Wrt. alerting.
 * Mirror SNMP exporter: "?target=..."
 * Authentication.
 * Options as CLI args.
 * Pass config to handler.
 */

#[tokio::main]
async fn main() {
    let config = read_config();
    run_server(config).await;
}

fn read_config() -> Config {
    let mut config = Config {
        http_port: Config::DEFAULT_HTTP_PORT,
        http_path: Config::DEFAULT_HTTP_PATH.to_owned(),
        log_requests_console: Config::DEFAULT_LOG_REQUESTS_CONSOLE,
    };

    if let Ok(http_port_str) = env::var("HTTP_PORT") {
        if let Ok(http_port) = http_port_str.parse::<u16>() {
            config.http_port = http_port;
        }
    }
    if let Ok(http_path) = env::var("HTTP_PATH") {
        if http_path.starts_with("/") {
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

/*
 * Web server stuff
 */

async fn run_server(config: Config) {
    let endpoint = SocketAddr::new(IpAddr::V6(Ipv6Addr::UNSPECIFIED), config.http_port);
    let service_maker = make_service_fn(move |conn:  &AddrStream| {
        let config = config.clone();
        let remote_addr = conn.remote_addr();
        async move {
            Ok::<_, Infallible>(service_fn(move |request: Request<Body>| {
                entrypoint(config.clone(), request, remote_addr)
            }))
        }
    });
    let server = Server::bind(&endpoint).serve(service_maker);

    println!("Listening on: http://{}", endpoint);
    if let Err(err) = server.await {
        eprintln!("Server error: {}", err);
    }
}

async fn entrypoint(config: Config, request: Request<Body>, remote_addr: SocketAddr) -> Result<Response<Body>, Infallible> {
    let query_args: HashMap<String, String> = form_urlencoded::parse(request.uri().query().unwrap_or("").as_bytes()).into_owned().collect();

    let metrics_path = &config.http_path;
    let is_method_get = request.method() == &Method::GET;
    let path = request.uri().path();
    let response_res: Result<Response<Body>, Infallible>;
    if path == "/" {
        if is_method_get {
            response_res = endpoint_home(&config);
        } else {
            response_res = endpoint_method_not_allowed();
        }
    } else if path == metrics_path {
        if is_method_get {
            response_res = endpoint_metrics(&config, &query_args).await;
        } else {
            response_res = endpoint_method_not_allowed();
        }
    } else {
        response_res = endpoint_not_found();
    }

    log_request(&config, &request, &remote_addr, &response_res);

    response_res
}

fn endpoint_home(config: &Config) -> Result<Response<Body>, Infallible> {
    let mut content = String::new();
    content.push_str("Prometheus NUT Exporter by HON95\n");
    content.push_str("\n");
    content.push_str("Project: https://github.com/HON95/prometheus-nut-exporter\n");
    content.push_str("\n");
    content.push_str(&format!("Usage: {}?target=<target>\n", config.http_path));

    let mut response = Response::new(Body::empty());
    *response.body_mut()= Body::from(content.into_bytes());
    Ok(response)
}

async fn endpoint_metrics(config: &Config, query_args: &HashMap<String, String>) -> Result<Response<Body>, Infallible> {
    let target = query_args.get("target").map_or("", |s| s.as_str());

    let mut content = String::new();
    let mut status = StatusCode::OK;

    if target.len() > 0 {
        content.push_str(&format!("Metrics!\n\nTarget: {}\n", target));
        let result_res = scrape_nut(target).await;
        if let Ok(result) = result_res {
            content.push_str(&result);
        } else if let Err(error) = result_res {
            content.push_str(&error);
            status = StatusCode::SERVICE_UNAVAILABLE;
        }
    } else {
        content.push_str(&format!("Missing target.\n\nUsage: {}?target=<target>\n", config.http_path));
        status = StatusCode::BAD_REQUEST;
    }

    let mut response = Response::new(Body::empty());
    *response.body_mut() = Body::from(content.into_bytes());
    *response.status_mut() = status;
    Ok(response)
}

fn endpoint_not_found() -> Result<Response<Body>, Infallible> {
    let mut response = Response::new(Body::from("Not found\n"));
    *response.status_mut() = StatusCode::NOT_FOUND;
    Ok(response)
}

fn endpoint_method_not_allowed() -> Result<Response<Body>, Infallible> {
    let mut response = Response::new(Body::from("Method not allowed\n"));
    *response.status_mut() = StatusCode::METHOD_NOT_ALLOWED;
    Ok(response)
}

fn log_request(config: &Config, request: &Request<Body>, remote_addr: &SocketAddr, response_res: &Result<Response<Body>, Infallible>) {
    if config.log_requests_console {
        // ISO 8601 timestamp
        let local_time = Local::now().format("%+");
        let status = response_res.as_ref().map_or("Error".to_owned(), |res| res.status().to_string());
        println!("{time} {client} {method} \"{path}\" {status}",
            time=local_time, client=remote_addr, method=request.method(), path=request.uri().path(), status=status);
    }
}

/*
 * NUT client stuff
 */

async fn scrape_nut(target: &str) -> Result<String, String> {
    // TODO connect, query UPSes, query vars for each UPS
    // TODO transform vars to pretty metrics with HELP, TYPE, UNIT
    // TODO build OpenMetrics content

    let raw_stream_res = TcpStream::connect(target).await;
    if let Err(err) = raw_stream_res {
        return Err("Failed to connect to target.".to_owned());
    }
    let mut stream = BufReader::new(raw_stream_res.unwrap());

    let vars_res = scrape_nut_vars(&mut stream).await;
    if let Err(err) = vars_res {
        return Err("Failed to communicate with target.".to_owned());
    }
    let vars = vars_res.unwrap();

    let content = build_openmetrics_content(vars);

    Ok(content)
}

async fn scrape_nut_vars(stream: &mut BufReader<TcpStream>) -> Result<Vec<UpsVar>, Error> {
    const RAW_UPS_PATTERN: &str = r#"^UPS\s+(?P<ups>[\S]+)\s+"(?P<desc>[^"]*)"$"#;
    const RAW_VAR_PATTERN: &str = r#"^VAR\s+(?P<ups>[\S]+)\s+(?P<var>[\S]+)\s+"(?P<val>[^"]*)"$"#;
    lazy_static! {
        static ref UPS_PATTERN: Regex = Regex::new(RAW_UPS_PATTERN).unwrap();
        static ref VAR_PATTERN: Regex = Regex::new(RAW_VAR_PATTERN).unwrap();
    }

    let mut upses: Vec<String> = Vec::new();
    let mut vars: Vec<UpsVar> = Vec::new();

    // Query UPSes
    stream.write_all(b"LIST UPS\n").await?;
    while let Some(line) = stream.lines().next_line().await? {
        if line.starts_with("BEGIN") {
            continue;
        }
        if line.starts_with("END") {
            break;
        }
        let captures_opt = UPS_PATTERN.captures(&line);
        match captures_opt {
            Some(captures) => {
                let ups = captures["ups"].to_owned();
                let desc = captures["desc"].to_owned();
                upses.push(ups.clone());
                vars.push(UpsVar {ups: ups.clone(), variable: UPS_DESCRIPTION_PSEUDOVAR.to_owned(), value: desc.clone()});
            },
            None => {
                continue;
            },
        }
    }

    // Query VARs
    for ups in upses {
        stream.write_all(format!("LIST VAR {}\n", ups).as_bytes()).await?;
        while let Some(line) = stream.lines().next_line().await? {
            if line.starts_with("BEGIN") {
                continue;
            }
            if line.starts_with("END") {
                break;
            }
            let captures_opt = VAR_PATTERN.captures(&line);
            match captures_opt {
                Some(captures) => {
                    let ups = captures["ups"].to_owned();
                    let var = captures["var"].to_owned();
                    let val = captures["val"].to_owned();
                    vars.push(UpsVar {ups: ups, variable: var, value: val});
                },
                None => {
                    continue;
                },
            }
        }
    }

    Ok(vars)
}

fn build_openmetrics_content(vars: Vec<UpsVar>) -> String {
    // TODO

    let mut content: String = String::new();
    for var in vars {
        content.push_str(&format!("{} {} \"{}\"\n", var.ups, var.variable, var.value));
    }
    content
}