use std::fmt::Write as _;
use std::collections::{HashMap, HashSet};

use crate::meta::APP_VERSION;
use crate::metrics::{EXPORTER_INFO_METRIC, Metric, METRIC_NAMES, METRICS, OLD_SERVER_INFO_METRIC, SERVER_INFO_METRIC, UPS_DESCRIPTION_PSEUDOVAR, UPS_INFO_METRIC, UPS_STATUS_ELEMENTS, UPS_STATUS_METRIC, UpsVarMap, VAR_METRICS, VarMap, VarTransform};

pub fn build_openmetrics_content(upses: &UpsVarMap, nut_version: &str) -> String {
    // Use vec for stable ordering of metrics within a metric family
    let mut metric_lines: HashMap<String, Vec<String>> = METRICS.keys().map(|m| ((*m).to_owned(), Vec::new())).collect();

    // Exporter and server metadata
    metric_lines.get_mut(EXPORTER_INFO_METRIC.metric).unwrap().push(print_exporter_info_metric());
    metric_lines.get_mut(SERVER_INFO_METRIC.metric).unwrap().push(print_server_info_metric(nut_version));
    metric_lines.get_mut(OLD_SERVER_INFO_METRIC.metric).unwrap().push(print_old_server_info_metric(nut_version));

    // Generate metric lines for all vars for all UPSes
    for (ups, vars) in upses.iter() {
        // UPS metadata
        metric_lines.get_mut(UPS_INFO_METRIC.metric).unwrap().push(print_ups_info_metric(ups, vars));
        metric_lines.get_mut(UPS_INFO_METRIC.metric).unwrap().append(&mut print_ups_status_metrics(ups, vars));
        // UPS vars
        for (var, val) in vars.iter() {
            if let Some(metrics) = VAR_METRICS.get(var.as_str()) {
                for metric in metrics {
                    if let Some(var_line) = print_basic_var_metric(ups, val, metric) {
                        metric_lines.get_mut(metric.metric).unwrap().push(var_line);
                    }
                }
            }
        }
    }

    // Print metric info and then all dimensions together
    // Use METRIC_NAMES vec for stable ordering of metric families
    let mut builder: String = String::new();
    for metric_name in METRIC_NAMES.iter() {
        let metric = METRICS[metric_name];
        if let Some(lines) = metric_lines.get(metric.metric) {
            if !lines.is_empty() {
                builder.push_str(&print_metric_metadata(metric));
                builder.push_str(&lines.concat());
            }
        }
    }
    builder.push_str("# EOF\n");

    builder
}

fn print_metric_metadata(metric: &Metric) -> String {
    let mut builder: String = String::new();
    let _ = writeln!(builder, "# TYPE {} {}", metric.metric, metric.type_);
    let _ = writeln!(builder, "# UNIT {} {}", metric.metric, metric.unit);
    if !metric.nut_var.is_empty() {
        let _ = writeln!(builder, "# HELP {} {} (\"{}\")", metric.metric, metric.help, metric.nut_var);
    } else {
        let _ = writeln!(builder, "# HELP {} {}", metric.metric, metric.help);
    }

    builder
}

fn print_exporter_info_metric() -> String {
    let metric = EXPORTER_INFO_METRIC;
    format!("{metric}{{version=\"{version}\"}} 1\n", metric=metric.metric, version=escape_om(APP_VERSION))
}

fn print_server_info_metric(nut_version: &str) -> String {
    let metric = SERVER_INFO_METRIC;
    format!("{metric}{{version=\"{version}\"}} 1\n", metric=metric.metric, version=escape_om(nut_version))
}

fn print_old_server_info_metric(nut_version: &str) -> String {
    let metric = OLD_SERVER_INFO_METRIC;
    format!("{metric}{{version=\"{version}\"}} 1\n", metric=metric.metric, version=escape_om(nut_version))
}

fn print_ups_info_metric(ups: &str, vars: &VarMap) -> String {
    let metric = UPS_INFO_METRIC;

    let mut labels_str = String::new();
    let _ = write!(labels_str, "ups=\"{}\"", escape_om(ups));
    let mut add_var_label = |name: &str, var: &str| {
        if let Some(value) = vars.get(var) {
            let _ = write!(labels_str, ",{}=\"{}\"", escape_om(name), escape_om(value));
        }
    };

    add_var_label("description", UPS_DESCRIPTION_PSEUDOVAR);
    add_var_label("description2", "device.description");
    add_var_label("device_type", "device.type");
    add_var_label("location", "device.location");
    add_var_label("manufacturer", "device.mfr");
    add_var_label("manufacturing_date", "device.mfr.date");
    add_var_label("model", "device.model");
    add_var_label("battery_type", "battery.type");
    add_var_label("driver", "driver.name");
    add_var_label("driver_version", "driver.version");
    add_var_label("driver_version_internal", "driver.version.internal");
    add_var_label("driver_version_data", "driver.version.data");
    add_var_label("usb_vendor_id", "ups.vendorid");
    add_var_label("usb_product_id", "ups.productid");
    add_var_label("ups_firmware", "ups.firmware");
    add_var_label("ups_type", "ups.type");
    // Deprecated
    add_var_label("type", "device.type");
    add_var_label("nut_version", "driver.version");

    format!("{}{{{}}} 1\n",metric.metric, labels_str)
}

fn print_ups_status_metrics(ups: &str, vars: &VarMap) -> Vec<String> {
    let metric = UPS_STATUS_METRIC;
    let mut lines: Vec<String> = Vec::new();

    let status_raw = match vars.get(metric.nut_var) {
        Some(x) => x,
        None => return lines,
    };
    let statuses: HashSet<&str> = HashSet::from_iter(status_raw.split(' '));

    for state in UPS_STATUS_ELEMENTS.iter() {
        let value_num = match statuses.contains(state) { false => 0, true => 1 };
        lines.push(format!("{metric}{{ups=\"{ups}\",status=\"{state}\"}} {value}\n", ups=ups, metric=metric.metric, state=state, value=value_num));
    }

    lines
}

fn print_basic_var_metric(ups: &str, value: &str, metric: &Metric) -> Option<String> {
    let result_value: f64 = match metric.var_transform {
        VarTransform::None => {
            match value.parse::<f64>() {
                Ok(val) => val,
                Err(_) => return None,
            }
        },
        VarTransform::Percentage => {
            let num_value = match value.parse::<f64>() {
                Ok(val) => val,
                Err(_) => return None,
            };
            num_value / 100f64
        },
        VarTransform::BeeperStatus => {
            match value {
                "enabled" => 1f64,
                "disabled" => 2f64,
                "muted" => 3f64,
                _ => 0f64,
            }
        },
        VarTransform::OldUpsStatus => {
            // Remove the second component if present ("LB" etc.)
            let value_start = value.split_once(' ').map_or(value, |x| x.0);
            match value_start {
                "OL" => 1f64,
                "OB" => 2f64,
                "LB" => 3f64,
                _ => 0f64,
            }
        },
    };

    // Make sure floats always contains a decimal point and that ints never do
    let result_str = match metric.is_integer {
        true => format!("{:.0}", result_value),
        false => format!("{:.17}", result_value),
    };

    Some(format!("{metric}{{ups=\"{ups}\"}} {value}\n", metric=metric.metric, ups=escape_om(ups), value=result_str))
}

fn escape_om(raw_text: &str) -> String {
    raw_text.chars().map(|c| match c {
        '\n' => r#"\n"#.to_string(),
        '"' => r#"\""#.to_string(),
        '\\' => r#"\\"#.to_string(),
        _ => c.to_string(),
    }).collect()
}
