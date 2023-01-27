pub fn voltage_to_temp(voltage: u32) -> i32 {
    let voltage = voltage as i32;
    if voltage > 2363 {
        0
    } else if voltage > 1352 {
        // 0 - 33
        76 - (voltage * 32 / 1000)
    } else if voltage > 971 {
        // 34 - 48
        87 - voltage * 40 / 1000
    } else if voltage > 664 {
        // 50 - 65
        98 - voltage * 50 / 1000
    } else if voltage > 411 {
        // 67 - 83
        112 - voltage * 70 / 1000
    } else {
        // 84 - 110 good approximation, then diverges
        139 - voltage * 130 / 1000
    }
}

#[cfg(test)]
mod test {
    use crate::voltage_to_temp::voltage_to_temp;

    #[test]
    fn temperature_is_converted_correctly() {
        let input = [
            2420, 2392, 2363, 2334, 2305, 2275, 2245, 2215, 2184, 2153, 2122, 2091, 2060, 2028,
            1997, 1965, 1934, 1902, 1870, 1838, 1807, 1775, 1744, 1712, 1681, 1650, 1619, 1588,
            1558, 1528, 1498, 1468, 1438, 1409, 1380, 1352, 1324, 1296, 1268, 1241, 1215, 1188,
            1162, 1137, 1112, 1087, 1063, 1039, 1015, 992, 971, 948, 926, 905, 884, 863, 843, 823,
            804, 785, 767, 749, 731, 714, 697, 680, 664, 648, 633, 618, 603, 589, 575, 561, 548,
            535, 522, 510, 497, 486, 474, 463, 452, 441, 431, 421,
        ];
        let expected_temperature = [
            0, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22,
            23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 42, 43, 44, 45,
            45, 46, 47, 48, 50, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 62, 63, 64, 65, 67, 68,
            69, 70, 71, 72, 73, 74, 75, 76, 76, 77, 78, 79, 80, 81, 81, 82, 83, 84,
        ];
        for i in 0..85 {
            let actual = voltage_to_temp(input[i]);
            let expected = expected_temperature[i];
            assert_eq!(
                0 <= expected - actual && expected - actual <= 1,
                true,
                "Failed for {}: diff: {}, expected: {}, actual: {}",
                input[i],
                expected - actual,
                expected,
                actual
            );
        }
    }
}
