/// Basically eyes perceive lightness changes in a non-linear way.
/// This function fills an array of 101 values with
///
pub fn fill_pwm_duty_cycle_values(duties: &mut [u16; 101], min: u16, max: u16) {
    for i in 0..=100 {
        let max = (max - min) as u32;

        let duty: u32 = if i < 8 {
            max * i as u32 / 903
        } else {
            // (i+16)/116)^3
            max * (i as u32 + 16) / 116 * (i as u32 + 16) / 116 * (i as u32 + 16) / 116
        };
        duties[i] = (duty as u16) + min;
    }
}

#[cfg(test)]
mod test {
    use crate::perceived_light_math::fill_pwm_duty_cycle_values;

    #[test]
    fn calculate_pwm() {
        let expected = [
            86, 87, 89, 91, 92, 94, 96, 97, 99, 101, 102, 104, 107, 109, 112, 114, 117, 120, 123,
            127, 131, 134, 139, 143, 147, 152, 157, 162, 168, 174, 179, 186, 193, 199, 207, 214,
            221, 229, 238, 246, 255, 265, 275, 284, 294, 305, 316, 327, 339, 351, 364, 377, 390,
            403, 417, 432, 447, 462, 478, 494, 511, 528, 545, 563, 581, 600, 619, 639, 659, 680,
            702, 723, 746, 768, 792, 815, 839, 865, 890, 916, 943, 969, 997, 1025, 1054, 1083,
            1113, 1144, 1175, 1207, 1239, 1272, 1305, 1340, 1374, 1410, 1446, 1482, 1520, 1559,
            1599,
        ];
        let mut duties: [u16; 101] = [0; 101];
        fill_pwm_duty_cycle_values(&mut duties, 86, 1599);
        assert_eq!(duties, expected);
    }
}
