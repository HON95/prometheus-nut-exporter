use std::collections::HashMap;
use std::convert::Infallible;
use std::fmt::Write as _;
use std::net::{SocketAddr};

use hyper::{Body, Method, Request, Response, Server, StatusCode};
use hyper::service::{make_service_fn, service_fn};
use hyper::server::conn::AddrStream;
use lazy_static::lazy_static;
use regex::Regex;
use tokio::sync::broadcast::Receiver;
use url::form_urlencoded;

use crate::meta::{APP_NAME, APP_AUTHOR, APP_VERSION};
use crate::common::ErrorResult;
use crate::config::Config;
use crate::nut_client::scrape_nut;
use crate::openmetrics_builder::build_openmetrics_content;

const CONTENT_TYPE_TEXT: &str = "text/plain; charset=UTF-8";
const CONTENT_TYPE_OPENMETRICS: &str = "application/openmetrics-text; version=1.0.0; charset=UTF-8";
const CONTENT_TYPE_OPENMETRICS_BASE: &str = "application/openmetrics-text";

pub async fn run_server(config: Config, mut shutdown_channel: Receiver<bool>) {
    // Bind to endpoint
    let endpoint = SocketAddr::new(config.http_address, config.http_port);
    log::info!("Binding to endpoint: http://{}", endpoint);
    let server_builder = match Server::try_bind(&endpoint) {
        Ok(builder) => builder,
        Err(err) => {
            log::error!("Server failed to bind to endpoint: {}", err);
            return;
        },
    };

    // Setup server
    let shutdown_future = async {
        shutdown_channel.recv().await.unwrap();
    };
    let config1 = config.clone();
    let service_maker = make_service_fn(move |conn:  &AddrStream| {
        let config2 = config1.clone();
        let remote_addr = conn.remote_addr();
        async move {
            Ok::<_, Infallible>(service_fn(move |request: Request<Body>| {
                entrypoint(config2.clone(), request, remote_addr)
            }))
        }
    });
    let server_task = server_builder.serve(service_maker).with_graceful_shutdown(shutdown_future);

    // Run server
    if let Err(err) = server_task.await {
        log::error!("Server error: {}", err);
    }
}

async fn entrypoint(config: Config, request: Request<Body>, remote_addr: SocketAddr) -> Result<Response<Body>, Infallible> {
    log::trace!("HTTP request from: {}", remote_addr);
    log::trace!("HTTP request URL: {}", request.uri().path());

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
            response = endpoint_metrics(&config, &request).await;
        } else {
            response = endpoint_method_not_allowed();
        }
    } else {
        response = endpoint_not_found();
    }

    // Log request to console
    log::debug!("Request: {} {} {} {}", remote_addr, request.method(), request.uri().path(), response.status().to_string());

    Ok(response)
}

fn endpoint_home(config: &Config) -> Response<Body> {
    let mut content = String::new();
    let _ = writeln!(content, "{} version {} by {}.", APP_NAME, APP_VERSION, APP_AUTHOR);
    let _ = writeln!(content);
    let _ = writeln!(content, "Usage: {}?target=<target>", config.http_path);

    Response::builder().status(StatusCode::OK).body(Body::from(content)).unwrap()
}

fn endpoint_not_found() -> Response<Body> {
    Response::builder().status(StatusCode::NOT_FOUND).body(Body::from("Not found\n")).unwrap()
}

fn endpoint_method_not_allowed() -> Response<Body> {
    Response::builder().status(StatusCode::METHOD_NOT_ALLOWED).body(Body::from("Method not allowed\n")).unwrap()
}

async fn endpoint_metrics(config: &Config, request: &Request<Body>) -> Response<Body> {
    // Check for and parse target
    let usage_message = format!("Usage: {}?target=<target>", config.http_path);
    let target = match parse_target(request) {
        Ok(target) => target,
        Err(err) => return Response::builder().status(StatusCode::BAD_REQUEST).body(Body::from(format!("{}\n\n{}", err, usage_message))).unwrap(),
    };

    // Try to scrape NUT server
    let (upses, nut_version) = match scrape_nut(&target).await {
        Ok(x) =>  x,
        Err(err) => return Response::builder().status(StatusCode::SERVICE_UNAVAILABLE).body(Body::from(err.to_string())).unwrap(),
    };

    // Generate OpenMetrics output
    let content = build_openmetrics_content(&upses, &nut_version);

    // Set content type
    let mut content_type = CONTENT_TYPE_TEXT;
    if let Some(accept_header) = request.headers().get("accept") {
        if let Ok(accept_str) = accept_header.to_str() {
            if accept_str.contains(CONTENT_TYPE_OPENMETRICS_BASE) {
                content_type = CONTENT_TYPE_OPENMETRICS;
            }
        }
    }

    Response::builder().status(StatusCode::OK).header("Content-Type", content_type).body(Body::from(content)).unwrap()
}

fn parse_target(request: &Request<Body>) -> ErrorResult<String> {
    lazy_static! {
        // Match domain, IPv4 address or IPv6 addres, with optional port number
        static ref TARGET_PATTERN: Regex = Regex::new(r#"^(?P<host>\[[^\]]+\]|[^:]+)(?::(?P<port>[0-9]+))?$"#).unwrap();
    }

    let query_args: HashMap<String, String> = form_urlencoded::parse(request.uri().query().unwrap_or("").as_bytes()).into_owned().collect();
    let target_raw = match query_args.get("target") {
        Some(target_raw) => target_raw,
        None => return Err("Missing target.".into()),
    };

    let default_port = Config::DEFAULT_NUT_PORT.to_string();
    let target = match TARGET_PATTERN.captures(target_raw) {
        Some(captures) => {
            let host = captures.name("host").unwrap().as_str();
            let port = match captures.name("port") {
                Some(port) => port.as_str(),
                None => default_port.as_str(),
            };
            format!("{}:{}", host, port)
        },
        None => return Err("Malformed list element for VAR list query.".into()),
    };

    Ok(target)
}
