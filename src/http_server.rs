use std::collections::HashMap;
use std::convert::Infallible;
use std::net::{IpAddr, Ipv6Addr, SocketAddr};

use chrono::Local;
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use hyper::service::{make_service_fn, service_fn};
use hyper::server::conn::AddrStream;
use url::form_urlencoded;

use crate::common::{PROJECT_TITLE, PROJECT_URL};
use crate::config::Config;
use crate::nut_client::scrape_nut_to_openmetrics;

pub async fn run_server(config: Config) {
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
    content.push_str(&format!("{}\n", PROJECT_TITLE));
    content.push_str("\n");
    content.push_str(&format!("Project: {}\n", PROJECT_URL));
    content.push_str("\n");
    content.push_str(&format!("Usage: {}?target=<target>\n", config.http_path));

    let mut response = Response::new(Body::empty());
    *response.body_mut()= Body::from(content.into_bytes());

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

async fn endpoint_metrics(config: &Config, query_args: &HashMap<String, String>) -> Response<Body> {
    let empty_str = "".to_owned();
    let target = query_args.get("target").unwrap_or(&empty_str);

    let mut content = String::new();
    let mut status = StatusCode::OK;

    if !target.is_empty() {
        let result_res = scrape_nut_to_openmetrics(&config, target).await;
        match result_res {
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

fn log_request(config: &Config, request: &Request<Body>, remote_addr: &SocketAddr, response: &Response<Body>) {
    if config.log_requests_console {
        // ISO 8601 timestamp
        let local_time = Local::now().format("%+");
        let status = response.status().to_string();
        println!("{time} {client} {method} \"{path}\" {status}",
            time=local_time, client=remote_addr, method=request.method(), path=request.uri().path(), status=status);
    }
}
