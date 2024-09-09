use std::collections::HashMap;

use lazy_static::lazy_static;
use regex::Regex;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;

use crate::common::ErrorResult;
use crate::config;
use crate::metrics::{NutVersion, UPS_DESCRIPTION_PSEUDOVAR, UpsVarMap, VarMap};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum NutQueryListState {
    Initial,
    Begun,
    Ended,
    Malformed,
    Error,
}

pub async fn scrape_nut(target: &str) -> ErrorResult<(UpsVarMap, NutVersion)> {
    log::trace!("Connecting to NUT server: {}", target);
    let raw_stream = match TcpStream::connect(target).await {
        Ok(val) => val,
        Err(err) => return Err(format!("Failed to connect to target: {}", err).into()),
    };
    let mut stream = BufReader::new(raw_stream);

    match scrape_nut_upses(&mut stream).await {
        Ok(val) => Ok(val),
        Err(err) => Err(format!("Failed to communicate with target: {}", err).into()),
    }
}

async fn scrape_nut_upses(stream: &mut BufReader<TcpStream>) -> ErrorResult<(UpsVarMap, NutVersion)> {
    let mut upses: UpsVarMap = HashMap::new();
    let mut nut_version: NutVersion = "".to_owned();

    query_nut_version(stream, &mut nut_version).await?;
    query_nut_upses(stream, &mut upses).await?;
    query_nut_vars(stream, &mut upses).await?;

    Ok((upses, nut_version))
}

async fn query_nut_version(stream: &mut BufReader<TcpStream>, nut_version: &mut NutVersion) -> ErrorResult<()> {
    lazy_static! {
        static ref VERSION_PATTERN: Regex = Regex::new(r#"upsd (?P<version>.+) -"#).unwrap();
    }

    stream.write_all(b"VER\n").await?;
    log::trace!("NUT query sent: {}", "VER");
    if let Some(line) = stream.lines().next_line().await? {
        log::trace!("NUT query received: {}", line);
        let captures_opt = VERSION_PATTERN.captures(&line);
        match captures_opt {
            Some(captures) => {
                *nut_version = captures["version"].to_owned();
            },
            None => {
                return Err("Failed get NUT version from NUT query. Not a NUT server?".into());
            },
        }
    }

    Ok(())
}

async fn query_nut_upses(stream: &mut BufReader<TcpStream>, upses: &mut UpsVarMap) -> ErrorResult<()> {
    lazy_static! {
        static ref UPS_PATTERN: Regex = Regex::new(r#"^UPS\s+(?P<ups>[\S]+)\s+"(?P<desc>[^"]*)"$"#).unwrap();
    }

    let line_consumer = |line: &str| {
        let captures_opt = UPS_PATTERN.captures(line);
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

    query_nut_list(stream, "LIST UPS", line_consumer).await?;

    Ok(())
}

async fn query_nut_vars(stream: &mut BufReader<TcpStream>, upses: &mut UpsVarMap) -> ErrorResult<()> {
    lazy_static! {
        static ref VAR_PATTERN: Regex = Regex::new(r#"^VAR\s+(?P<ups>[\S]+)\s+(?P<var>[\S]+)\s+"(?P<val>[^"]*)"$"#).unwrap();
    }

    for (ups, vars) in upses.iter_mut() {
        {
            let line_consumer = |line: &str| {
                let captures_opt = VAR_PATTERN.captures(line);
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

            query_nut_list(stream, format!("LIST VAR {}", ups).as_str(), line_consumer).await?;
        }

        override_nut_values(vars).await;
    }

    Ok(())
}


async fn override_nut_values(vars: &mut HashMap<String, String>) {
    let config = config::read_config();

    // For UPS systems where ups.power is not available, but ups.load and ups.realpower.nominal are.
    // Provide a calculated value for ups.power with the following algorithm:
    //     power (W) = ( load(%) / 100.00 ) * nominal power (W)
    if config.ups_power_from_load_percentage {
        if !( vars.contains_key("ups.power") ) && vars.contains_key("ups.load") && vars.contains_key("ups.realpower.nominal") {
            if let (Ok(load), Ok(nominal_power)) = (
                vars.get("ups.load").unwrap().parse::<f64>(),
                vars.get("ups.realpower.nominal").unwrap().parse::<f64>(),
            ) {
                let calculated_power = (load / 100.0) * nominal_power;
                vars.insert(String::from("ups.power"), calculated_power.to_string());
            } else {
                log::error!("Unable to parse ups.load or ups.realpower.nominal as floats, skipping calculation of ups.power from these values");
            }
        }
    }
}

async fn query_nut_list<F>(stream: &mut BufReader<TcpStream>, query: &str, mut line_consumer: F) -> ErrorResult<()>
        where F: FnMut(&str) -> ErrorResult<()> + Send {
    let query_line = format!("{}\n", query);
    stream.write_all(query_line.as_bytes()).await?;
    log::trace!("NUT query sent: {}", query);
    let mut query_state = NutQueryListState::Initial;
    let mut nut_error_message = "".to_owned();
    while let Some(line) = stream.lines().next_line().await? {
        log::trace!("NUT query received: {}", line);

        // Empty line
        if line.is_empty() {
            // Skip line
            continue;
        }
        // Start of list
        if line.starts_with("BEGIN ") {
            if query_state == NutQueryListState::Initial {
                query_state = NutQueryListState::Begun;
                // Continue with list
                continue;
            } else {
                // Wrong order
                query_state = NutQueryListState::Malformed;
                break;
            }
        }
        // End of list
        if line.starts_with("END ") {
            if query_state == NutQueryListState::Begun {
                // End list
                query_state = NutQueryListState::Ended;
                break;
            } else {
                // Wrong order
                query_state = NutQueryListState::Malformed;
                break;
            }
        }
        // Error
        if line.starts_with("ERR ") {
            query_state = NutQueryListState::Error;
            nut_error_message = line.strip_prefix("ERR ").unwrap().to_owned();
            break;
        }

        // Structural error if content outside BEGIN-END section
        if query_state != NutQueryListState::Begun {
            query_state = NutQueryListState::Malformed;
            break;
        }

        // Within BEGIN-END so feed line to consumer
        line_consumer(&line)?;
    }

    // Check if the list didn't finish traversal when no error was encountered
    if query_state != NutQueryListState::Ended && query_state != NutQueryListState::Error {
        query_state = NutQueryListState::Malformed;
    }

    // Check if error or malformed
    if query_state == NutQueryListState::Error {
        return Err(format!("Received error for query \"{}\": {}", query, nut_error_message).into());
    }
    if query_state == NutQueryListState::Malformed {
        return Err(format!("Malformed list for query \"{}\".", query).into());
    }

    Ok(())
}
