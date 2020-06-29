use std::collections::HashMap;
use std::convert::Infallible;
use std::env;
use std::error::Error;
use std::fs;
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

type SimpleResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug, Clone)]
struct Config {
    version: String,
    http_port: u16,
    http_path: String,
    log_requests_console: bool,
}

impl Config {
    const DEFAULT_VERSION: &'static str = "0.0.0";
    const DEFAULT_HTTP_PORT: u16 = 9995;
    const DEFAULT_HTTP_PATH: &'static str = "/nut";
    const DEFAULT_LOG_REQUESTS_CONSOLE: bool = false;
}

const VERSION_FILE: &str = "VERSION";

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum NutQueryListState {
    Initial,
    Begun,
    Ended,
}

type VarMap = HashMap<String, String>;
type UpsVarMap = HashMap<String, VarMap>;
type NutVersion = String;

const UPS_DESCRIPTION_PSEUDOVAR: &str = "description";

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
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

const EXPORTER_INFO_METRIC: Metric = Metric {
    metric: "nut_exporter_info",
    help: "Metadata about the exporter.",
    type_: "gauge",
    unit: "",
    nut_var: "",
    var_transform: VarTransform::None,
};

const NUT_INFO_METRIC: Metric = Metric {
    metric: "nut_info",
    help: "Metadata about the NUT server.",
    type_: "gauge",
    unit: "",
    nut_var: "",
    var_transform: VarTransform::None,
};

const UPS_INFO_METRIC: Metric = Metric {
    metric: "nut_ups_info",
    help: "Metadata about the UPS.",
    type_: "gauge",
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
        map.insert(EXPORTER_INFO_METRIC.metric, &EXPORTER_INFO_METRIC);
        map.insert(NUT_INFO_METRIC.metric, &NUT_INFO_METRIC);
        map.insert(UPS_INFO_METRIC.metric, &UPS_INFO_METRIC);
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

fn read_version_file(config: &mut Config) {
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
    let response: Response<Body>;
    if path == "/" {
        if is_method_get {
            response = endpoint_home(&config);
        } else {
            response = endpoint_method_not_allowed();
        }
    } else if path == metrics_path {
        if is_method_get {
            response = endpoint_metrics(&config, &query_args).await;
        } else {
            response = endpoint_method_not_allowed();
        }
    } else {
        response = endpoint_not_found();
    }

    log_request(&config, &request, &remote_addr, &response);

    Ok(response)
}

fn endpoint_home(config: &Config) -> Response<Body> {
    let mut content = String::new();
    content.push_str("Prometheus NUT Exporter by HON95\n");
    content.push_str("\n");
    content.push_str("Project: https://github.com/HON95/prometheus-nut-exporter\n");
    content.push_str("\n");
    content.push_str(&format!("Usage: {}?target=<target>\n", config.http_path));

    let mut response = Response::new(Body::empty());
    *response.body_mut()= Body::from(content.into_bytes());

    response
}

async fn endpoint_metrics(config: &Config, query_args: &HashMap<String, String>) -> Response<Body> {
    let empty_str = "".to_owned();
    let target = query_args.get("target").unwrap_or(&empty_str);

    let mut content = String::new();
    let mut status = StatusCode::OK;

    if !target.is_empty() {
        match scrape_nut(&config, target).await {
            Ok(result) => {
                content.push_str(result.as_str());
            },
            Err(error) => {
                content.push_str(error.to_string().as_str());
                status = StatusCode::SERVICE_UNAVAILABLE;
            },
        }
    } else {
        content.push_str(&format!("Missing target.\n\nUsage: {}?target=<target>\n", config.http_path));
        status = StatusCode::BAD_REQUEST;
    }

    let mut response = Response::new(Body::empty());
    *response.body_mut() = Body::from(content.into_bytes());
    *response.status_mut() = status;

    response
}

fn endpoint_not_found() -> Response<Body> {
    let mut response = Response::new(Body::from("Not found\n"));
    *response.status_mut() = StatusCode::NOT_FOUND;

    response
}

fn endpoint_method_not_allowed() -> Response<Body> {
    let mut response = Response::new(Body::from("Method not allowed\n"));
    *response.status_mut() = StatusCode::METHOD_NOT_ALLOWED;

    response
}

fn log_request(config: &Config, request: &Request<Body>, remote_addr: &SocketAddr, response: &Response<Body>) {
    if config.log_requests_console {
        // ISO 8601 timestamp
        let local_time = Local::now().format("%+");
        let status = response.status().to_string();
        println!("{time} {client} {method} \"{path}\" {status}",
            time=local_time, client=remote_addr, method=request.method(), path=request.uri().path(), status=status);
    }
}

/*
 * NUT client stuff
 */

async fn scrape_nut(config: &Config, target: &str) -> Result<String, Box<dyn Error>> {
    let raw_stream = match TcpStream::connect(target).await {
        Ok(val) => val,
        Err(_) => return Err("Failed to connect to target.".into()),
    };
    let mut stream = BufReader::new(raw_stream);

    let (upses, nut_version) = match scrape_nut_upses(&mut stream).await {
        Ok(val) => val,
        Err(err) => return Err(format!("Failed to communicate with target.\n\n{}", err).into()),
    };

    let content = build_openmetrics_content(&config, &upses, &nut_version);

    Ok(content)
}

async fn scrape_nut_upses(mut stream: &mut BufReader<TcpStream>) -> Result<(UpsVarMap, NutVersion), Box<dyn Error>> {
    let mut upses: UpsVarMap = HashMap::new();
    let mut nut_version: NutVersion = "".to_owned();
    query_nut_version(&mut stream, &mut nut_version).await?;
    query_nut_upses(&mut stream, &mut upses).await?;
    query_nut_vars(&mut stream, &mut upses).await?;

    Ok((upses, nut_version))
}

async fn query_nut_version(stream: &mut BufReader<TcpStream>, nut_version: &mut NutVersion) -> SimpleResult<()> {
    const RAW_VERSION_PATTERN: &str = r#"(?P<version>\d+\.\d+\.\d+)"#;
    lazy_static! {
        static ref VERSION_PATTERN: Regex = Regex::new(RAW_VERSION_PATTERN).unwrap();
    }

    stream.write_all(b"VER\n").await?;
    if let Some(line) = stream.lines().next_line().await? {
        let captures_opt = VERSION_PATTERN.captures(&line);
        match captures_opt {
            Some(captures) => {
                *nut_version = captures["version"].to_owned();
            },
            None => {
                return Err("Failed get NUT version from NUT query.".into());
            },
        }
    }

    Ok(())
}

async fn query_nut_upses(mut stream: &mut BufReader<TcpStream>, upses: &mut UpsVarMap) -> SimpleResult<()> {
    const RAW_UPS_PATTERN: &str = r#"^UPS\s+(?P<ups>[\S]+)\s+"(?P<desc>[^"]*)"$"#;
    lazy_static! {
        static ref UPS_PATTERN: Regex = Regex::new(RAW_UPS_PATTERN).unwrap();
    }

    let line_consumer = |line: &str| {
        let captures_opt = UPS_PATTERN.captures(&line);
        match captures_opt {
            Some(captures) => {
                let ups = captures["ups"].to_owned();
                let desc = captures["desc"].to_owned();
                let mut vars: VarMap = HashMap::new();
                vars.insert(UPS_DESCRIPTION_PSEUDOVAR.to_owned(), desc);
                upses.insert(ups, vars);
            },
            None => {
                return Err("Malformed list element for UPS list query.".into());
            },
        }

        Ok(())
    };

    query_nut_list(&mut stream, "UPS", line_consumer).await?;

    Ok(())
}

async fn query_nut_vars(mut stream: &mut BufReader<TcpStream>, upses: &mut UpsVarMap) -> SimpleResult<()> {
    const RAW_VAR_PATTERN: &str = r#"^VAR\s+(?P<ups>[\S]+)\s+(?P<var>[\S]+)\s+"(?P<val>[^"]*)"$"#;
    lazy_static! {
        static ref VAR_PATTERN: Regex = Regex::new(RAW_VAR_PATTERN).unwrap();
    }

    for (ups, vars) in upses.iter_mut() {
        let line_consumer = |line: &str| {
            let captures_opt = VAR_PATTERN.captures(&line);
            match captures_opt {
                Some(captures) => {
                    let variable = captures["var"].to_owned();
                    let value = captures["val"].to_owned();
                    vars.insert(variable, value);
                },
                None => {
                    return Err("Malformed list element for VAR list query.".into());
                },
            }

            Ok(())
        };

        query_nut_list(&mut stream, format!("VAR {}\n", ups).as_str(), line_consumer).await?;
    }

    Ok(())
}

async fn query_nut_list<F>(stream: &mut BufReader<TcpStream>, query_param: &str, mut line_consumer: F) -> SimpleResult<()>
        where F: FnMut(&str) -> SimpleResult<()> + Send {
    let query = format!("LIST {}\n", query_param);
    stream.write_all(query.as_bytes()).await?;
    let mut query_state = NutQueryListState::Initial;
    while let Some(line) = stream.lines().next_line().await? {
        // Start of list
        if line.starts_with("BEGIN") {
            if query_state == NutQueryListState::Initial {
                query_state = NutQueryListState::Begun;
                // Continue with list
                continue;
            } else {
                // Wrong order
                break;
            }
        }
        // End of list
        if line.starts_with("END") {
            if query_state == NutQueryListState::Begun {
                query_state = NutQueryListState::Ended;
                // End list
                break;
            } else {
                // Wrong order
                break;
            }
        }

        // Structural error if content outside BEGIN-END section
        if query_state != NutQueryListState::Begun {
            break;
        }

        // Feed line to consumer
        line_consumer(&line)?;
    }

    // Check if a list was traversed or if the content was malformed
    if query_state != NutQueryListState::Ended {
        return Err(format!("Malformed list for NUT query \"{}\".", query).into());
    }

    Ok(())
}

fn build_openmetrics_content(config: &Config, upses: &UpsVarMap, nut_version: &str) -> String {
    let mut metric_lines: HashMap<String, Vec<String>> = METRICS.keys().map(|m| ((*m).to_owned(), Vec::new())).collect();

    // Exporter metadata
    let exporter_info_line = print_exporter_info_metric(config);
    metric_lines.get_mut(EXPORTER_INFO_METRIC.metric).unwrap().push(exporter_info_line);

    // NUT metadata
    let nut_info_line = print_nut_info_metric(nut_version);
    metric_lines.get_mut(NUT_INFO_METRIC.metric).unwrap().push(nut_info_line);

    // Generate metric lines for all vars for all UPSes
    for (ups, vars) in upses.iter() {
        // UPS metadata
        let ups_info_line = print_ups_info_metric(&ups, &vars);
        metric_lines.get_mut(UPS_INFO_METRIC.metric).unwrap().push(ups_info_line);
        // UPS vars
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

fn print_exporter_info_metric(config: &Config) -> String {
    let metric = EXPORTER_INFO_METRIC;

    format!("{metric}{{version=\"{version}\"}} 1\n", metric=metric.metric, version=config.version)
}

fn print_nut_info_metric(nut_version: &str) -> String {
    let metric = NUT_INFO_METRIC;

    format!("{metric}{{version=\"{version}\"}} 1\n", metric=metric.metric, version=nut_version)
}

fn print_ups_info_metric(ups: &str, vars: &VarMap) -> String {
    let metric = UPS_INFO_METRIC;
    let empty_str = "".to_owned();
    let battery_type = vars.get("battery.type").unwrap_or(&empty_str);
    let device_model = vars.get("device.model").unwrap_or(&empty_str);
    let driver = vars.get("driver.name").unwrap_or(&empty_str);

    format!(
        "{metric}{{ups=\"{ups}\",battery_type=\"{battery_type}\",device_model=\"{device_model}\",driver=\"{driver}\"}} 1\n",
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
