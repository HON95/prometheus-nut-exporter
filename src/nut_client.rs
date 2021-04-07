use std::collections::HashMap;
use std::error::Error;

use lazy_static::lazy_static;
use regex::Regex;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;

use crate::common::ErrorResult;
use crate::metrics::{NutVersion, UPS_DESCRIPTION_PSEUDOVAR, UpsVarMap, VarMap};
use crate::openmetrics_builder::build_openmetrics_content;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum NutQueryListState {
    Initial,
    Begun,
    Ended,
}

pub async fn scrape_nut_to_openmetrics(target: &str) -> Result<String, Box<dyn Error>> {
    let raw_stream = match TcpStream::connect(target).await {
        Ok(val) => val,
        Err(err) => return Err(format!("Failed to connect to target: {}", err).into()),
    };
    let mut stream = BufReader::new(raw_stream);

    let (upses, nut_version) = match scrape_nut_upses(&mut stream).await {
        Ok(val) => val,
        Err(err) => return Err(format!("Failed to communicate with target: {}", err).into()),
    };

    let content = build_openmetrics_content(&upses, &nut_version);

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

async fn query_nut_version(stream: &mut BufReader<TcpStream>, nut_version: &mut NutVersion) -> ErrorResult<()> {
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

async fn query_nut_upses(mut stream: &mut BufReader<TcpStream>, upses: &mut UpsVarMap) -> ErrorResult<()> {
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

async fn query_nut_vars(mut stream: &mut BufReader<TcpStream>, upses: &mut UpsVarMap) -> ErrorResult<()> {
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

async fn query_nut_list<F>(stream: &mut BufReader<TcpStream>, query_param: &str, mut line_consumer: F) -> ErrorResult<()>
        where F: FnMut(&str) -> ErrorResult<()> + Send {
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
