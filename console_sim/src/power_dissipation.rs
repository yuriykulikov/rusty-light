use light_control::battery_voltage_to_capacity::battery_voltage_to_capacity;

/// ## Given current temperature and led power, calculates radiator after (some) time period
///
/// Using Newton's law of cooling, cooling is proportional to temperature difference between
/// radiator and ambient temperature.
///
/// ### Assumptions
/// Assuming 20 degrees C ambient temperature
/// Assuming convection covers most of the cooling -> Newton's law of cooling
/// Ignoring LED efficiency drift
///
pub fn calculate_temperature(led_low: u32, led_high: u32, prev_temp: i32) -> i32 {
    let prev_temp = prev_temp as f64;
    let led_low = (led_low as f64) / 100.0;
    let led_high = (led_high as f64) / 100.0;
    let efficiency = 0.3;
    let max_power = 16f64;
    let ambient = 20.0;
    let wind_coefficent = 0.3;
    let generated_power = (1.0 - efficiency) * max_power * (led_low + led_high);
    let dissipated_power = wind_coefficent * (prev_temp - ambient);
    let diff = generated_power - dissipated_power;
    let new_temp: f64 = prev_temp + diff * 1.0;
    new_temp as i32
}

pub fn battery_capacity(capacity: u32) -> u32 {
    return (6000..8500)
        .step_by(100)
        .find(|voltage| battery_voltage_to_capacity(*voltage) >= capacity)
        .unwrap_or(0);
}
