use std::collections::HashMap;

use lazy_static::lazy_static;

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
    help: "Metadata about the UPS (e.g. model, battery type, location).",
    type_: "gauge",
    unit: "",
    nut_var: "",
    var_transform: VarTransform::None,
};

pub static BASIC_METRICS: [Metric; 36] = [
    // Status
    Metric {
        metric: "nut_status",
        help: "UPS status. Unknown (0), on line (1, \"OL\"), on battery (2, \"OB\"), or low battery (3, \"LB\").",
        type_: "gauge",
        unit: "",
        nut_var: "ups.status",
        var_transform: VarTransform::UpsStatus,
    },
    Metric {
        metric: "nut_beeper_status",
        help: "If the beeper is enabled. Unknown (0), enabled (1), disabled (2) or muted (3).",
        type_: "gauge",
        unit: "",
        nut_var: "ups.beeper.status",
        var_transform: VarTransform::BeeperStatus,
    },
    // Load, charge, runtime, uptime
    Metric {
        metric: "nut_load",
        help: "Load. (0-1)",
        type_: "gauge",
        unit: "",
        nut_var: "ups.load",
        var_transform: VarTransform::Percent,
    },
    Metric {
        metric: "nut_battery_charge",
        help: "Battery level. (0-1)",
        type_: "gauge",
        unit: "",
        nut_var: "battery.charge",
        var_transform: VarTransform::Percent,
    },
    Metric {
        metric: "nut_battery_charge_low",
        help: "Battery level threshold for low state. (0-1)",
        type_: "gauge",
        unit: "",
        nut_var: "battery.charge.low",
        var_transform: VarTransform::Percent,
    },
    Metric {
        metric: "nut_battery_charge_warning",
        help: "Battery level threshold for warning state. (0-1)",
        type_: "gauge",
        unit: "",
        nut_var: "battery.charge.warning",
        var_transform: VarTransform::Percent,
    },
    Metric {
        metric: "nut_battery_charge_restart",
        help: "Battery level threshold for restarting after power-off. (0-1)",
        type_: "gauge",
        unit: "",
        nut_var: "battery.charge.restart",
        var_transform: VarTransform::Percent,
    },
    Metric {
        metric: "nut_battery_runtime_seconds",
        help: "Battery runtime.",
        type_: "gauge",
        unit: "s",
        nut_var: "battery.runtime",
        var_transform: VarTransform::None,
    },
    Metric {
        metric: "nut_battery_runtime_low_seconds",
        help: "Battery runtime threshold for state low.",
        type_: "gauge",
        unit: "s",
        nut_var: "battery.runtime.low",
        var_transform: VarTransform::None,
    },
    Metric {
        metric: "nut_battery_runtime_restart_seconds",
        help: "Battery runtime threshold for restart after power-off.",
        type_: "gauge",
        unit: "s",
        nut_var: "battery.runtime.restart",
        var_transform: VarTransform::None,
    },
    Metric {
        metric: "nut_uptime_seconds",
        help: "Device uptime.",
        type_: "gauge",
        unit: "s",
        nut_var: "device.uptime",
        var_transform: VarTransform::None,
    },
    // Battery
    Metric {
        metric: "nut_battery_voltage_volts",
        help: "Battery voltage.",
        type_: "gauge",
        unit: "V",
        nut_var: "battery.voltage",
        var_transform: VarTransform::None,
    },
    Metric {
        metric: "nut_battery_voltage_nominal_volts",
        help: "Battery voltage (nominal).",
        type_: "gauge",
        unit: "V",
        nut_var: "battery.voltage.nominal",
        var_transform: VarTransform::None,
    },
    Metric {
        metric: "nut_battery_temperature_celsius",
        help: "Battery temperature.",
        type_: "gauge",
        unit: "degrees C",
        nut_var: "battery.temperature",
        var_transform: VarTransform::None,
    },
    // Input
    Metric {
        metric: "nut_input_voltage_volts",
        help: "Input voltage.",
        type_: "gauge",
        unit: "V",
        nut_var: "input.voltage",
        var_transform: VarTransform::None,
    },
    Metric {
        metric: "nut_input_voltage_nominal_volts",
        help: "Input voltage (nominal).",
        type_: "gauge",
        unit: "V",
        nut_var: "input.voltage.nominal",
        var_transform: VarTransform::None,
    },
    Metric {
        metric: "nut_input_voltage_minimum_volts",
        help: "Input voltage (minimum seen).",
        type_: "gauge",
        unit: "V",
        nut_var: "input.voltage.minimum",
        var_transform: VarTransform::None,
    },
    Metric {
        metric: "nut_input_voltage_maximum_volts",
        help: "Input voltage (maximum seen).",
        type_: "gauge",
        unit: "V",
        nut_var: "input.voltage.maximum",
        var_transform: VarTransform::None,
    },
    Metric {
        metric: "nut_input_transfer_low_volts",
        help: "Input lower transfer threshold.",
        type_: "gauge",
        unit: "V",
        nut_var: "input.transfer.low",
        var_transform: VarTransform::None,
    },
    Metric {
        metric: "nut_input_transfer_high_volts",
        help: "Input upper transfer threshold.",
        type_: "gauge",
        unit: "V",
        nut_var: "input.transfer.high",
        var_transform: VarTransform::None,
    },
    Metric {
        metric: "nut_input_current_amperes",
        help: "Input current.",
        type_: "gauge",
        unit: "A",
        nut_var: "input.current",
        var_transform: VarTransform::None,
    },
    Metric {
        metric: "nut_input_current_nominal_amperes",
        help: "Input current (nominal).",
        type_: "gauge",
        unit: "A",
        nut_var: "input.current.nominal",
        var_transform: VarTransform::None,
    },
    Metric {
        metric: "nut_input_frequency_hertz",
        help: "Input frequency.",
        type_: "gauge",
        unit: "Hz",
        nut_var: "input.frequency",
        var_transform: VarTransform::None,
    },
    Metric {
        metric: "nut_input_frequency_nominal_hertz",
        help: "Input frequency (nominal).",
        type_: "gauge",
        unit: "Hz",
        nut_var: "input.frequency.nominal",
        var_transform: VarTransform::None,
    },
    Metric {
        metric: "nut_input_frequency_low_hertz",
        help: "Input frequency (low).",
        type_: "gauge",
        unit: "Hz",
        nut_var: "input.frequency.low",
        var_transform: VarTransform::None,
    },
    Metric {
        metric: "nut_input_frequency_high_hertz",
        help: "Input frequency (high).",
        type_: "gauge",
        unit: "Hz",
        nut_var: "input.frequency.high",
        var_transform: VarTransform::None,
    },
    // Output
    Metric {
        metric: "nut_output_voltage_volts",
        help: "Output voltage.",
        type_: "gauge",
        unit: "V",
        nut_var: "output.voltage",
        var_transform: VarTransform::None,
    },
    Metric {
        metric: "nut_output_voltage_nominal_volts",
        help: "Output voltage (nominal).",
        type_: "gauge",
        unit: "V",
        nut_var: "output.voltage.nominal",
        var_transform: VarTransform::None,
    },
    Metric {
        metric: "nut_output_current_amperes",
        help: "Output current.",
        type_: "gauge",
        unit: "A",
        nut_var: "output.current",
        var_transform: VarTransform::None,
    },
    Metric {
        metric: "nut_output_current_nominal_amperes",
        help: "Output current (nominal).",
        type_: "gauge",
        unit: "A",
        nut_var: "output.current.nominal",
        var_transform: VarTransform::None,
    },
    Metric {
        metric: "nut_output_frequency_hertz",
        help: "Output frequency.",
        type_: "gauge",
        unit: "Hz",
        nut_var: "output.frequency",
        var_transform: VarTransform::None,
    },
    Metric {
        metric: "nut_output_frequency_nominal_hertz",
        help: "Output frequency (nominal).",
        type_: "gauge",
        unit: "Hz",
        nut_var: "output.frequency.nominal",
        var_transform: VarTransform::None,
    },
    // UPS power
    Metric {
        metric: "nut_power_watts",
        help: "Apparent power.",
        type_: "gauge",
        unit: "W",
        nut_var: "ups.power",
        var_transform: VarTransform::None,
    },
    Metric {
        metric: "nut_power_nominal_watts",
        help: "Apparent power (nominal).",
        type_: "gauge",
        unit: "W",
        nut_var: "ups.power.nominal",
        var_transform: VarTransform::None,
    },
    Metric {
        metric: "nut_real_power_watts",
        help: "Real power.",
        type_: "gauge",
        unit: "W",
        nut_var: "ups.realpower",
        var_transform: VarTransform::None,
    },
    Metric {
        metric: "nut_real_power_nominal_watts",
        help: "Real power (nominal).",
        type_: "gauge",
        unit: "W",
        nut_var: "ups.realpower.nominal",
        var_transform: VarTransform::None,
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

// Print metrics as Markdown table.
pub fn print_metrics() {
    println!("| Metric | NUT Var | Unit | Description |");
    println!("| - | - | - | - |");
    for metric in (*METRICS).values() {
        let row = format!("| `{}` | `{}` | `{}` | {} |", metric.metric, metric.nut_var, metric.unit, metric.help).replace("``", "");
        println!("{}", row)
    }
}
