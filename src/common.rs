use std::collections::HashMap;
use std::error::Error;

use lazy_static::lazy_static;

pub const PROJECT_TITLE: &str = "Prometheus NUT Exporter by HON95";
pub const PROJECT_URL: &str = "https://github.com/HON95/prometheus-nut-exporter";

pub type SimpleResult<T> = Result<T, Box<dyn Error>>;

pub type VarMap = HashMap<String, String>;
pub type UpsVarMap = HashMap<String, VarMap>;
pub type NutVersion = String;

pub const UPS_DESCRIPTION_PSEUDOVAR: &str = "description";

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum VarTransform {
    None,
    Percent,
    BeeperStatus,
    UpsStatus,
}

#[derive(Debug, Clone)]
pub struct Metric {
    pub metric: &'static str,
    pub help: &'static str,
    pub type_: &'static str,
    pub unit: &'static str,
    pub nut_var: &'static str,
    pub var_transform: VarTransform,
}

pub const EXPORTER_INFO_METRIC: Metric = Metric {
    metric: "nut_exporter_info",
    help: "Metadata about the exporter.",
    type_: "gauge",
    unit: "",
    nut_var: "",
    var_transform: VarTransform::None,
};

pub const NUT_INFO_METRIC: Metric = Metric {
    metric: "nut_info",
    help: "Metadata about the NUT server.",
    type_: "gauge",
    unit: "",
    nut_var: "",
    var_transform: VarTransform::None,
};

pub const UPS_INFO_METRIC: Metric = Metric {
    metric: "nut_ups_info",
    help: "Metadata about the UPS.",
    type_: "gauge",
    unit: "",
    nut_var: "",
    var_transform: VarTransform::None,
};

pub static BASIC_METRICS: [Metric; 9] = [
    Metric {
        metric: "nut_battery_charge",
        help: "Battery level. (0-1)",
        type_: "gauge",
        unit: "",
        nut_var: "battery.charge",
        var_transform: VarTransform::Percent,
    },
    Metric {
        metric: "nut_battery_type",
        help: "Get battery type",
        type_: "gauge",
        unit: "",
        nut_var: "battery.type",
        var_transform: VarTransform::None,
    },
    Metric {
        metric: "nut_battery_charge_low",
        help: "Get battery charge low",
        type_: "gauge",
        unit: "",
        nut_var: "battery.charge.low",
        var_transform: VarTransform::None,
    },
    Metric {
        metric: "nut_battery_charge_warning",
        help: "Get battery charge warning",
        type_: "gauge",
        unit: "",
        nut_var: "battery.charge.warning",
        var_transform: VarTransform::None,
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
        metric: "nut_device_mfr",
        help: "Get device manufacture",
        type_: "gauge",
        unit: "",
        nut_var: "device.mfr",
        var_transform: VarTransform::None,
    },
    Metric {
        metric: "nut_device_model",
        help: "Get device model",
        type_: "gauge",
        unit: "",
        nut_var: "device.model",
        var_transform: VarTransform::None,
    },
    Metric {
        metric: "nut_device_part",
        help: "Get device part",
        type_: "gauge",
        unit: "",
        nut_var: "device.part",
        var_transform: VarTransform::None,
    },
    Metric {
        metric: "nut_device_type",
        help: "Get device type",
        type_: "gauge",
        unit: "",
        nut_var: "device.type",
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
        metric: "nut_input_frequency",
        help: "Input frequency.",
        type_: "gauge",
        unit: "Hz",
        nut_var: "input.frequency",
        var_transform: VarTransform::None,
    },
    Metric {
        metric: "nut_input_frequency_nominal",
        help: "Input frequency nominal.",
        type_: "gauge",
        unit: "Hz",
        nut_var: "input.voltage.nominal",
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
        metric: "nut_output_frequency",
        help: "Output frequency.",
        type_: "gauge",
        unit: "Hz",
        nut_var: "output.frequency",
        var_transform: VarTransform::None,
    },
    Metric {
        metric: "nut_output_frequency_nominal",
        help: "Output frequency nominal.",
        type_: "gauge",
        unit: "Hz",
        nut_var: "output.voltage.nominal",
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
        metric: "nut_power_nominal_watts",
        help: "Nominal value of real power.",
        type_: "gauge",
        unit: "W",
        nut_var: "ups.power.nominal",
        var_transform: VarTransform::None,
    },
    Metric {
        metric: "nut_power_watts",
        help: "Value of real power.",
        type_: "gauge",
        unit: "W",
        nut_var: "ups.power",
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
    pub static ref METRICS: HashMap<&'static str, &'static Metric> = {
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
    pub static ref VAR_METRICS: HashMap<&'static str, &'static Metric> = {
        let mut map: HashMap<&'static str, &'static Metric> = HashMap::new();
        for metric in BASIC_METRICS.iter() {
            map.insert(metric.nut_var, &metric);
        }

        map
    };
}
