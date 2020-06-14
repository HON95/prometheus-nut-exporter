use std::collections::HashMap;
use std::fmt;
use std::io::{BufRead, Write};
use std::net::{IpAddr, Ipv6Addr, SocketAddr};
use std::net::{TcpListener, TcpStream};
use std::time::Duration;

use bufstream::BufStream;
use chrono::Local;
use lazy_static::lazy_static;
use regex::Regex;

/*
 * TODO
 * - User-friendly / endpoint
 * - OpenMetrics /metrics endpoint
 * - Concurrent requests
 * - Caching after compiling deps does not build app
 * - Check if unabailable, tmp and perm. Wrt. alerting.
 * - Mirror SNMP exporter.
 * - Authentication.
 * - Options as CLI args.
 */

const HTTP_CONTENT_TYPE_TEXT_PLAIN: &str = "text/plain";
const HTTP_STATUS_OK: &str = "200 OK";
const HTTP_STATUS_BAD_REQUEST: &str = "400 Bad Request";
const HTTP_STATUS_NOT_FOUND: &str = "404 Not Found";
const HTTP_STATUS_METHOD_NOT_ALLOWED: &str = "405 Method Not Allowed";

// Bodyless HTTP request.
struct SimpleHttpRequest {
    method: String,
    target: String,
    path: String,
    query: String,
    query_args: HashMap<String, String>,
    http_version: String,
}

impl fmt::Display for SimpleHttpRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{} \"{}\"", self.method, self.target)
    }
}

// Message states of a bodyless HTTP message.
#[derive(PartialEq)]
enum SimpleHttpMessagePart {
    STATUS,
    HEADERS,
    END,
}

impl fmt::Display for SimpleHttpMessagePart {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            SimpleHttpMessagePart::STATUS => write!(formatter, "status"),
            SimpleHttpMessagePart::HEADERS => write!(formatter, "headers"),
            SimpleHttpMessagePart::END => write!(formatter, "end"),
        }
    }
}

#[derive(Debug, Clone)]
struct HttpParseError {
    message: String,
}

impl HttpParseError {
    pub fn new(message: String) -> HttpParseError {
        HttpParseError {
            message: message,
        }
    }
}

impl fmt::Display for HttpParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "Malformed request: {}", self.message)
    }
}

static PORT: u16 = 9999;
static CLIENT_CONN_TIMEOUT_S: u64 = 5;
static LOG_REQUESTS_CONSOLE: bool = true;

fn main() {
    run_server();
}

fn run_server() {
    let endpoint = SocketAddr::new(IpAddr::V6(Ipv6Addr::UNSPECIFIED), PORT);
    let listener: TcpListener = TcpListener::bind(endpoint).expect(&format!("Error binding to endpoint {}.", endpoint));
    println!("Listening on: {}\n", endpoint);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => handle_client(stream),
            Err(err) => log_request("UNKNOWN".to_owned(), format!("Connection error: {}", err)),
        }
    }
}

fn handle_client(tcp_stream: TcpStream) {
    let peer_addr = tcp_stream.peer_addr().unwrap().to_string();

    // Set RW timeouts
    let timeout: Option<Duration> = Some(Duration::new(CLIENT_CONN_TIMEOUT_S, 0));
    tcp_stream.set_read_timeout(timeout).unwrap();
    tcp_stream.set_write_timeout(timeout).unwrap();

    let mut stream = BufStream::new(tcp_stream);

    // Parse request
    let request_res = parse_http_request(&mut stream);

    // Handle request
    let response;
    match request_res {
        Ok(request) => {
            log_request(peer_addr, request.to_string());
            response = handle_request(request);
        },
        Err(err) => {
            log_request(peer_addr, format!("Request error: {}", err.message));
            response = build_http_response(HTTP_STATUS_BAD_REQUEST, None, Some(err.message));
        },
    }

    send_http_response(&mut stream, response);
}

fn log_request(peer_addr: String, message: String) {
    if LOG_REQUESTS_CONSOLE {
        // ISO 8601
        let local_time = Local::now().format("%+").to_string();
        println!("{} {} {}", local_time, peer_addr, message);
    }
}

fn parse_http_request(stream: &mut BufStream<TcpStream>) -> Result<SimpleHttpRequest, HttpParseError> {
    let mut request: SimpleHttpRequest = SimpleHttpRequest {
        method: "".to_owned(),
        target: "".to_owned(),
        path: "".to_owned(),
        query: "".to_owned(),
        query_args: HashMap::new(),
        http_version: "".to_owned(),
    };

    let mut part = SimpleHttpMessagePart::STATUS;
    loop {
        let mut full_line = String::new();
        let size_res = stream.read_line(&mut full_line);
        match size_res {
            Ok(0) => {
                return Err(HttpParseError::new("Received unexpected EOF while reading HTTP request.".to_owned()));
            },
            Ok(_) => {
                let line = full_line.trim().to_owned();
                parse_http_request_line(&mut request, &mut part, line)?;
                if part == SimpleHttpMessagePart::END {
                    break;
                }
            },
            Err(err) => {
                return Err(HttpParseError::new(format!("Failed to read line: {}", err)));
            },
        }
    }

    Ok(request)
}

fn parse_http_request_line(request: &mut SimpleHttpRequest, part: &mut SimpleHttpMessagePart, line: String) -> Result<(), HttpParseError> {
    // End of head
    if line.len() == 0 {
        *part = SimpleHttpMessagePart::END;
        return Ok(());
    }

    match part {
        SimpleHttpMessagePart::STATUS => {
            parse_http_request_status(request, line)?;
            *part = SimpleHttpMessagePart::HEADERS;
        },
        SimpleHttpMessagePart::HEADERS => {
            // Ignore all headers
        },
        SimpleHttpMessagePart::END => {
        },
    }

    Ok(())
}

fn parse_http_request_status(request: &mut SimpleHttpRequest, line: String) -> Result<(), HttpParseError> {
    const RAW_PATTERN: &str = r"^(?P<method>GET|HEAD|POST|PUT|DELETE|CONNECT|OPTIONS|TRACE|PATCH)\s+(?P<target>[^\s]+)\s+(?P<http_version>HTTP/(?:1\.0|1\.1))$";
    lazy_static! {
        static ref PATTERN: Regex = Regex::new(RAW_PATTERN).unwrap();
    }

    let captures_opt = PATTERN.captures(&line);
    match captures_opt {
        Some(captures) => {
            request.method = captures["method"].to_owned();
            request.target = captures["target"].to_owned();
            request.http_version = captures["http_version"].to_owned();
            parse_request_target(request)?;
        },
        None => {
            return Err(HttpParseError::new("Malformed status line.".to_owned()));
        },
    }

    Ok(())
}

fn parse_request_target(request: &mut SimpleHttpRequest) -> Result<(), HttpParseError> {
    const RAW_URL_PATTERN: &str = r"^(?P<path>[^\?]*)(?:\?(?P<query>.+))?$";
    const RAW_PAIR_PATTERN: &str = r"^(?P<key>[^=]+)(?:=(?P<value>.+))?$";
    lazy_static! {
        static ref URL_PATTERN: Regex = Regex::new(RAW_URL_PATTERN).unwrap();
        static ref PAIR_PATTERN: Regex = Regex::new(RAW_PAIR_PATTERN).unwrap();
    }

    // Parse path and query
    let captures_opt = URL_PATTERN.captures(&request.target);
    match captures_opt {
        Some(captures) => {
            request.path = captures["path"].to_owned();
            // Optional
            request.query = captures.name("query").map_or("", |m| m.as_str()).to_owned();
        },
        None => {
            return Err(HttpParseError::new("Malformed URL.".to_owned()));
        },
    }

    // Parse query key-value pairs
    let mut query_iter = request.query.split("&");
    while let Some(pair) = query_iter.next() {
        if pair.len() == 0 {
            continue;
        }
        let captures_opt = PAIR_PATTERN.captures(&pair);
        match captures_opt {
            Some(captures) => {
                let key = captures["key"].to_owned();
                // Optional
                let value = captures.name("value").map_or("", |m| m.as_str()).to_owned();
                request.query_args.insert(key, value);
            },
            None => {
                return Err(HttpParseError::new("Malformed URL.".to_owned()));
            },
        }
    }

    Ok(())
}

fn handle_request(request: SimpleHttpRequest) -> String {
    if request.method != "GET" {
        return build_http_response(HTTP_STATUS_METHOD_NOT_ALLOWED, None, None);
    }

    match request.path.as_str() {
        "/" => return build_http_response_home(),
        "/metrics" => return build_http_response_metrics(request.query_args),
        _ => return build_http_response_not_found(),
    }
}

fn build_http_response_home() -> String {
    let text = "\
    Prometheus NUT Exporter by HON95 \n\n\
    Project: https://github.com/HON95/prometheus-nut-exporter \n\n\
    Usage: /metrics?a=b&c=d\
    ".to_owned();
    build_http_response(HTTP_STATUS_OK, None, Some(text))
}

fn build_http_response_metrics(_arguments: HashMap<String, String>) -> String {
    let text = "\
    TODO
    ".to_owned();
    build_http_response(HTTP_STATUS_OK, None, Some(text))
}

fn build_http_response_not_found() -> String {
    let text = "Not found".to_owned();
    build_http_response(HTTP_STATUS_NOT_FOUND, None, Some(text))
}

fn build_http_response(status: &str, content_type_opt: Option<&str>, content_opt: Option<String>) -> String {
    let content_type = content_type_opt.unwrap_or(&HTTP_CONTENT_TYPE_TEXT_PLAIN);
    let content = content_opt.unwrap_or("".to_owned());
    format!(
        "HTTP/1.1 {}\r\n\
        Content-Type: {}; charset=UTF-8\r\n\
        \r\n\
        {}\r\n",
        status, content_type, content
    )
}

fn send_http_response(stream: &mut BufStream<TcpStream>, response: String) {
    let result = stream.write_all(response.as_bytes());
    match result {
        Ok(_) => {
        },
        Err(err) => {
            eprintln!("Failed to send response: {}", err);
        }
    }
}
