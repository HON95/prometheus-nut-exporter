# Metrics

Copied from the source code.

| Metric | NUT var | Description | Unit |
| - | - | - | - |
| `nut_ups_info` | `` | Metadata about the UPS, including NUT description, device model, battery type and driver. | |
| `nut_battery_charge` | `battery.charge` | Battery level. (0-1) |  |
| `nut_battery_runtime_seconds` | `battery.runtime` | Seconds until battery runs out. | `s` |
| `nut_battery_volts` | `battery.voltage` | Battery voltage. | `V` |
| `nut_input_volts` | `input.voltage` | Input voltage. | `V` |
| `nut_output_volts` | `output.voltage` | Output voltage. | `V` |
| `nut_beeper_status` | `ups.beeper.status` | If the beeper is enabled. Unknown (0), enabled (1), disabled (2) or muted (3). | |
| `nut_load` | `ups.load` | Load. (0-1) | |
| `nut_realpower_nominal_watts` | `ups.realpower.nominal` | Nominal value of real power. | `W` |
| `nut_status` | `ups.status` | UPS status. Unknown (0), on line (OL) (1), on battery (OB) (2) or low battery (LB) (3). | `W` |
