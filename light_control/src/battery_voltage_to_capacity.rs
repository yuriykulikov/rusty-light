/// https://learn.adafruit.com/li-ion-and-lipoly-batteries/voltages
pub fn battery_voltage_to_capacity(battery_voltage_mv: u32) -> u32 {
    if battery_voltage_mv > 8300 {
        100
    } else if battery_voltage_mv >= 7700 {
        (battery_voltage_mv - 7700) * 50 / 1000 + 70
    } else if battery_voltage_mv >= 7460 {
        (battery_voltage_mv - 7460) * 120 / 1000 + 45
    } else if battery_voltage_mv >= 7360 {
        (battery_voltage_mv - 7360) * 240 / 1000 + 20
    } else if battery_voltage_mv >= 7280 {
        (battery_voltage_mv - 7280) * 90 / 1000 + 13
    } else {
        (battery_voltage_mv - 6000) * 8 / 1000
    }
}

#[cfg(test)]
mod test {
    use alloc::vec::Vec;

    use crate::battery_voltage_to_capacity::battery_voltage_to_capacity;

    #[test]
    fn percentages() {
        let voltages = [
            8400, 8200, 8100, 8000, 7900, 7800, 7700, 7640, 7600, 7540, 7500, 7460, 7440, 7420,
            7400, 7380, 7360, 7300, 7280, 7200, 6920, 6600, 6300, 6000,
        ];

        let expected = [
            100, 95, 90, 85, 80, 75, 70, 66, 61, 54, 49, 45, 39, 34, 29, 24, 20, 14, 13, 9, 7, 4,
            2, 0,
        ];
        let res: Vec<u32> = voltages
            .iter()
            .map(|it| battery_voltage_to_capacity(*it))
            .collect();
        assert_eq!(res, expected);
    }
}
