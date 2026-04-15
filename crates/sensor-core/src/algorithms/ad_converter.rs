pub struct AdConverter;

impl AdConverter {
    pub fn convert(
        ad_value: f64,
        zero_ad: f64,
        zero_physical: f64,
        scale_ad: f64,
        scale_physical: f64,
    ) -> f64 {
        let slope = (scale_physical - zero_physical) / (scale_ad - zero_ad);
        zero_physical + (ad_value - zero_ad) * slope
    }

    pub fn validate_calibration(zero_ad: f64, scale_ad: f64) -> Result<(), String> {
        if (scale_ad - zero_ad).abs() < f64::EPSILON {
            return Err(format!(
                "标定参数无效: scale_ad ({}) 不能等于 zero_ad ({})，会导致除零错误",
                scale_ad, zero_ad
            ));
        }
        Ok(())
    }

    pub fn convert_batch(
        ad_values: &[f64],
        zero_ad: f64,
        zero_physical: f64,
        scale_ad: f64,
        scale_physical: f64,
    ) -> Vec<f64> {
        let slope = (scale_physical - zero_physical) / (scale_ad - zero_ad);
        ad_values
            .iter()
            .map(|&ad| zero_physical + (ad - zero_ad) * slope)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_basic() {
        let result = AdConverter::convert(2047.5, 0.0, 0.0, 4095.0, 50.0);
        assert!((result - 25.0).abs() < 0.1);
    }

    #[test]
    fn test_convert_with_offset() {
        let result = AdConverter::convert(2050.0, 100.0, 5.0, 4000.0, 45.0);
        let expected = 5.0 + (2050.0 - 100.0) * (45.0 - 5.0) / (4000.0 - 100.0);
        assert!((result - expected).abs() < 0.01);
    }

    #[test]
    fn test_convert_zero_point() {
        let result = AdConverter::convert(0.0, 0.0, 0.0, 4095.0, 50.0);
        assert!((result - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_convert_scale_point() {
        let result = AdConverter::convert(4095.0, 0.0, 0.0, 4095.0, 50.0);
        assert!((result - 50.0).abs() < 0.01);
    }

    #[test]
    fn test_validate_calibration_valid() {
        assert!(AdConverter::validate_calibration(0.0, 4095.0).is_ok());
        assert!(AdConverter::validate_calibration(100.0, 4000.0).is_ok());
    }

    #[test]
    fn test_validate_calibration_invalid() {
        assert!(AdConverter::validate_calibration(100.0, 100.0).is_err());
        assert!(AdConverter::validate_calibration(2047.5, 2047.5).is_err());
    }

    #[test]
    fn test_convert_batch() {
        let ad_values = vec![0.0, 1023.75, 2047.5, 3071.25, 4095.0];
        let results = AdConverter::convert_batch(&ad_values, 0.0, 0.0, 4095.0, 50.0);

        assert_eq!(results.len(), 5);
        assert!((results[0] - 0.0).abs() < 0.1);
        assert!((results[1] - 12.5).abs() < 0.1);
        assert!((results[2] - 25.0).abs() < 0.1);
        assert!((results[3] - 37.5).abs() < 0.1);
        assert!((results[4] - 50.0).abs() < 0.1);
    }
}
