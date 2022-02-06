use std::collections::HashMap;

use crate::meta::APP_VERSION;
use crate::metrics::{EXPORTER_INFO_METRIC, Metric, METRICS, NUT_INFO_METRIC, UPS_DESCRIPTION_PSEUDOVAR, UPS_INFO_METRIC, UpsVarMap, VAR_METRICS, VarMap, VarTransform};

pub fn build_openmetrics_content(upses: &UpsVarMap, nut_version: &str) -> String {
    let mut metric_lines: HashMap<String, Vec<String>> = METRICS.keys().map(|m| ((*m).to_owned(), Vec::new())).collect();

    // Exporter metadata
    let exporter_info_line = print_exporter_info_metric();
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
            if let Some(metrics) = VAR_METRICS.get(var.as_str()) {
                for metric in metrics {
                    if let Some(var_line) = print_basic_var_metric(&ups, &val, &metric) {
                        metric_lines.get_mut(metric.metric).unwrap().push(var_line);
                    }
                }
            }
        }
    }

    // Print metric info and then all dimensions together
    let mut builder: String = String::new();
    for metric in METRICS.values() {
        if let Some(lines) = metric_lines.get(metric.metric) {
            if !lines.is_empty() {
                builder.push_str(&print_metric_info(&metric));
                builder.push_str(&lines.concat());
            }
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

fn print_exporter_info_metric() -> String {
    let metric = EXPORTER_INFO_METRIC;
    format!("{metric}{{version=\"{version}\"}} 1\n", metric=metric.metric, version=APP_VERSION)
}

fn print_nut_info_metric(nut_version: &str) -> String {
    let metric = NUT_INFO_METRIC;
    format!("{metric}{{version=\"{version}\"}} 1\n", metric=metric.metric, version=nut_version)
}

fn print_ups_info_metric(ups: &str, vars: &VarMap) -> String {
    let metric = UPS_INFO_METRIC;
    let empty_str = "".to_owned();

    let mut attributes = HashMap::new();
    attributes.insert("ups", ups);
    attributes.insert("description", vars.get(UPS_DESCRIPTION_PSEUDOVAR).unwrap_or(&empty_str));
    attributes.insert("description2", vars.get("device.description").unwrap_or(&empty_str));
    attributes.insert("location", vars.get("device.location").unwrap_or(&empty_str));
    attributes.insert("type", vars.get("device.type").unwrap_or(&empty_str));
    attributes.insert("manufacturer", vars.get("device.mfr").unwrap_or(&empty_str));
    attributes.insert("model", vars.get("device.model").unwrap_or(&empty_str));
    attributes.insert("battery_type", vars.get("battery.type").unwrap_or(&empty_str));
    attributes.insert("driver", vars.get("driver.name").unwrap_or(&empty_str));
    attributes.insert("nut_version", vars.get("driver.version").unwrap_or(&empty_str));
    attributes.insert("usb_vendor_id", vars.get("ups.vendorid").unwrap_or(&empty_str));
    attributes.insert("usb_product_id", vars.get("ups.productid").unwrap_or(&empty_str));
    attributes.insert("ups_firmware", vars.get("ups.firmware").unwrap_or(&empty_str));
    attributes.insert("ups_type", vars.get("ups.type").unwrap_or(&empty_str));

    let mut labels = String::new();
    for (key, value) in attributes {
        if !labels.is_empty() {
            labels.push(',');
        }
        labels.push_str(&format!("{}=\"{}\"", key, value));
    }

    format!("{}{{{}}} 1\n",metric.metric, labels)
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
