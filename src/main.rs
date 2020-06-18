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

type VarMap = HashMap<String, String>;
type UpsVarMap = HashMap<String, VarMap>;

const UPS_DESCRIPTION_PSEUDOVAR: &str = "description";

#[derive(Debug, Copy, Clone)]
enum VarTransform {
    None,
    Percent,
    BeeperStatus,
    UpsStatus,
}

#[derive(Debug, Clone)]
struct Metric {
    metric: &'static str,
    help: &'static str,
    type_: &'static str,
    unit: &'static str,
    nut_var: &'static str,
    var_transform: VarTransform,
}

const INFO_METRIC: Metric = Metric {
    metric: "nut_ups_info",
    help: "Metadata about the UPS.",
    type_: "counter",
    unit: "",
    nut_var: "",
    var_transform: VarTransform::None,
};

static BASIC_METRICS: [Metric; 9] = [
    Metric {
        metric: "nut_battery_charge",
        help: "Battery level. (0-1)",
        type_: "gauge",
        unit: "",
        nut_var: "battery.charge",
        var_transform: VarTransform::Percent,
    },
    Metric {
        metric: "nut_battery_runtime_seconds",
        help: "Seconds until battery runs out.",
        type_: "gauge",
        unit: "s",
        nut_var: "battery.runtime",
        var_transform: VarTransform::None,
    },
    Metric {
        metric: "nut_battery_volts",
        help: "Battery voltage.",
        type_: "gauge",
        unit: "V",
        nut_var: "battery.voltage",
        var_transform: VarTransform::None,
    },
    Metric {
        metric: "nut_input_volts",
        help: "Input voltage.",
        type_: "gauge",
        unit: "V",
        nut_var: "input.voltage",
        var_transform: VarTransform::None,
    },
    Metric {
        metric: "nut_output_volts",
        help: "Output voltage.",
        type_: "gauge",
        unit: "V",
        nut_var: "output.voltage",
        var_transform: VarTransform::None,
    },
    Metric {
        metric: "nut_beeper_status",
        help: "If the beeper is enabled. Unknown (0), enabled (1), disabled (2) or muted (3).",
        type_: "gauge",
        unit: "",
        nut_var: "ups.beeper.status",
        var_transform: VarTransform::BeeperStatus,
    },
    Metric {
        metric: "nut_load",
        help: "Load. (0-1)",
        type_: "gauge",
        unit: "",
        nut_var: "ups.load",
        var_transform: VarTransform::Percent,
    },
    Metric {
        metric: "nut_realpower_nominal_watts",
        help: "Nominal value of real power.",
        type_: "gauge",
        unit: "W",
        nut_var: "ups.realpower.nominal",
        var_transform: VarTransform::None,
    },
    Metric {
        metric: "nut_status",
        help: "UPS status. \
        Unknown (0), \
        on line (OL) (1), \
        on battery (OB) (2) or \
        low battery (LB) (3).",
        type_: "gauge",
        unit: "W",
        nut_var: "ups.status",
        var_transform: VarTransform::UpsStatus,
    },
];

lazy_static! {
    // Containes all metrics, indexed by metric name
    static ref METRICS: HashMap<&'static str, &'static Metric> = {
        let mut map: HashMap<&'static str, &'static Metric> = HashMap::new();
        map.insert(INFO_METRIC.metric, &INFO_METRIC);
        for metric in BASIC_METRICS.iter() {
            map.insert(metric.metric, &metric);
        }
        map
    };

    // Containes all metrics based on NUT vars, indexed by var
    static ref VAR_METRICS: HashMap<&'static str, &'static Metric> = {
        let mut map: HashMap<&'static str, &'static Metric> = HashMap::new();
        for metric in BASIC_METRICS.iter() {
            map.insert(metric.nut_var, &metric);
        }
        map
    };
}

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
    let is_method_get = request.method() == Method::GET;
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
    let empty_str = "".to_owned();
    let target = query_args.get("target").unwrap_or(&empty_str);

    let mut content = String::new();
    let mut status = StatusCode::OK;

    if !target.is_empty() {
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
    let raw_stream = match TcpStream::connect(target).await {
        Ok(val) => val,
        Err(_) => return Err("Failed to connect to target.".to_owned()),
    };
    let mut stream = BufReader::new(raw_stream);

    let upses = match scrape_nut_upses(&mut stream).await {
        Ok(val) => val,
        Err(_) => return Err("Failed to communicate with target.".to_owned()),
    };

    let content = build_openmetrics_content(&upses);

    Ok(content)
}

async fn scrape_nut_upses(mut stream: &mut BufReader<TcpStream>) -> Result<UpsVarMap, Error> {
    let mut upses: UpsVarMap = HashMap::new();
    query_nut_upses(&mut stream, &mut upses).await?;
    query_nut_vars(&mut stream, &mut upses).await?;

    Ok(upses)
}

async fn query_nut_upses(stream: &mut BufReader<TcpStream>, upses: &mut UpsVarMap) -> Result<(), Error> {
    const RAW_UPS_PATTERN: &str = r#"^UPS\s+(?P<ups>[\S]+)\s+"(?P<desc>[^"]*)"$"#;
    lazy_static! {
        static ref UPS_PATTERN: Regex = Regex::new(RAW_UPS_PATTERN).unwrap();
    }

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
                let mut vars: VarMap = HashMap::new();
                vars.insert(UPS_DESCRIPTION_PSEUDOVAR.to_owned(), desc.clone());
                upses.insert(ups.clone(), vars);
            },
            None => {
                continue;
            },
        }
    }

    Ok(())
}

async fn query_nut_vars(stream: &mut BufReader<TcpStream>, upses: &mut UpsVarMap) -> Result<(), Error> {
    const RAW_VAR_PATTERN: &str = r#"^VAR\s+(?P<ups>[\S]+)\s+(?P<var>[\S]+)\s+"(?P<val>[^"]*)"$"#;
    lazy_static! {
        static ref VAR_PATTERN: Regex = Regex::new(RAW_VAR_PATTERN).unwrap();
    }

    for (ups, vars) in upses.iter_mut() {
        stream.write_all(format!("LIST VAR {}\n", ups).as_bytes()).await?;
        while let Some(line) = stream.lines().next_line().await? {
            if line.starts_with("ERR") {
                break;
            }
            if line.starts_with("BEGIN") {
                continue;
            }
            if line.starts_with("END") {
                break;
            }
            let captures_opt = VAR_PATTERN.captures(&line);
            match captures_opt {
                Some(captures) => {
                    let variable = captures["var"].to_owned();
                    let value = captures["val"].to_owned();
                    vars.insert(variable.clone(), value.clone());
                },
                None => {
                    continue;
                },
            }
        }
    }

    Ok(())
}

fn build_openmetrics_content(upses: &UpsVarMap) -> String {
    let mut metric_lines: HashMap<String, Vec<String>> = METRICS.keys().map(|m| (m.to_string(), Vec::new())).collect();

    // Generate metric lines for all vars for all UPSes
    for (ups, vars) in upses.iter() {
        let info_line = print_ups_info_metric(&ups, &vars);
        metric_lines.get_mut(INFO_METRIC.metric).unwrap().push(info_line);
        for (var, val) in vars.iter() {
            if let Some(metric) = VAR_METRICS.get(var.as_str()) {
                if let Some(var_line) = print_basic_var_metric(&ups, &val, &metric) {
                    metric_lines.get_mut(metric.metric).unwrap().push(var_line);
                }
            }
        }
    }

    // Print metric info and then all dimensions together
    let mut builder: String = String::new();
    for metric in METRICS.values() {
        if let Some(lines) = metric_lines.get(metric.metric) {
            builder.push_str(&print_metric_info(&metric));
            builder.push_str(&lines.concat());
        }
    }

    builder
}

fn print_metric_info(metric: &Metric) -> String {
    let mut builder: String = String::new();
    if !metric.nut_var.is_empty() {
        builder.push_str(&format!("# HELP {} {} (\"{}\")\n", metric.metric, metric.help, metric.nut_var));
    } else {
        builder.push_str(&format!("# HELP {} {}\n", metric.metric, metric.help));
    }
    builder.push_str(&format!("# TYPE {} {}\n", metric.metric, metric.type_));
    builder.push_str(&format!("# UNIT {} {}\n", metric.metric, metric.unit));
    builder
}

fn print_ups_info_metric(ups: &str, vars: &VarMap) -> String {
    let metric = INFO_METRIC;
    let empty_str = "".to_owned();
    let battery_type = vars.get("battery.type").unwrap_or(&empty_str);
    let device_model = vars.get("device.model").unwrap_or(&empty_str);
    let driver = vars.get("driver.name").unwrap_or(&empty_str);
    
    format!(
        "{metric}{{ups=\"{ups}\",battery_type=\"{battery_type}\"device_model=\"{device_model}\"driver=\"{driver}\"}} 1\n",
        metric=metric.metric, ups=ups, battery_type=battery_type, device_model=device_model, driver=driver
    )
}

fn print_basic_var_metric(ups: &str, value: &str, metric: &Metric) -> Option<String> {
    let result_value: f64;
    match metric.var_transform {
        VarTransform::None => {
            result_value = match value.parse::<f64>() {
                Ok(val) => val,
                Err(_) => return None,
            };
        },
        VarTransform::Percent => {
            let num_value = match value.parse::<f64>() {
                Ok(val) => val,
                Err(_) => return None,
            };
            result_value = num_value / 100f64;
        },
        VarTransform::BeeperStatus => {
            result_value = match value {
                "enabled" => 1f64,
                "disabled" => 2f64,
                "muted" => 3f64,
                _ => 0f64,
            };
        },
        VarTransform::UpsStatus => {
            // Remove stuff we don't care about
            let value_start = value.splitn(2, ' ').next().unwrap();
            result_value = match value_start {
                "OL" => 1f64,
                "OB" => 2f64,
                "LB" => 3f64,
                _ => 0f64,
            };
        },
    }

    Some(format!("{metric}{{ups=\"{ups}\"}} {value}\n", metric=metric.metric, ups=ups, value=result_value))
}
