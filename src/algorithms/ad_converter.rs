/// AD 值转物理值转换器
/// 
/// 使用两点标定法进行线性转换
pub struct AdConverter;

impl AdConverter {
    /// AD 值转物理值通用公式
    /// 
    /// # 公式
    /// 物理值 = 零点物理值 + (AD值 - 零点AD) × (放大物理值 - 零点物理值) / (放大AD - 零点AD)
    /// 
    /// # 参数
    /// - `ad_value`: 当前 AD 采集值
    /// - `zero_ad`: 零点 AD 值（标定点1）
    /// - `zero_physical`: 零点物理值（标定点1）
    /// - `scale_ad`: 放大 AD 值（标定点2）
    /// - `scale_physical`: 放大物理值（标定点2）
    /// 
    /// # 返回
    /// 转换后的物理值
    /// 
    /// # 示例
    /// ```
    /// use qt_rust_demo::algorithms::ad_converter::AdConverter;
    /// 
    /// // 12位 AD (0-4095)，量程 0-50 吨
    /// let weight = AdConverter::convert(
    ///     2047.5,  // AD 中点
    ///     0.0,     // 零点 AD
    ///     0.0,     // 零点物理值
    ///     4095.0,  // 满量程 AD
    ///     50.0     // 满量程物理值
    /// );
    /// assert!((weight - 25.0).abs() < 0.1);
    /// ```
    pub fn convert(
        ad_value: f64,
        zero_ad: f64,
        zero_physical: f64,
        scale_ad: f64,
        scale_physical: f64,
    ) -> f64 {
        // 计算斜率（物理值变化量 / AD 值变化量）
        let slope = (scale_physical - zero_physical) / (scale_ad - zero_ad);
        
        // 线性转换
        zero_physical + (ad_value - zero_ad) * slope
    }
    
    /// 验证标定参数是否有效
    /// 
    /// # 参数
    /// - `zero_ad`: 零点 AD 值
    /// - `scale_ad`: 放大 AD 值
    /// 
    /// # 返回
    /// - `Ok(())`: 参数有效
    /// - `Err(String)`: 参数无效，包含错误描述
    pub fn validate_calibration(zero_ad: f64, scale_ad: f64) -> Result<(), String> {
        if (scale_ad - zero_ad).abs() < f64::EPSILON {
            return Err(format!(
                "标定参数无效: scale_ad ({}) 不能等于 zero_ad ({})，会导致除零错误",
                scale_ad, zero_ad
            ));
        }
        Ok(())
    }
    
    /// 批量转换 AD 值
    /// 
    /// # 参数
    /// - `ad_values`: AD 值数组
    /// - `zero_ad`: 零点 AD 值
    /// - `zero_physical`: 零点物理值
    /// - `scale_ad`: 放大 AD 值
    /// - `scale_physical`: 放大物理值
    /// 
    /// # 返回
    /// 转换后的物理值数组
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
        // 测试基本转换：0-4095 AD -> 0-50 吨
        let result = AdConverter::convert(2047.5, 0.0, 0.0, 4095.0, 50.0);
        assert!((result - 25.0).abs() < 0.1);
    }
    
    #[test]
    fn test_convert_with_offset() {
        // 测试带偏移的转换：100-4000 AD -> 5-45 吨
        let result = AdConverter::convert(2050.0, 100.0, 5.0, 4000.0, 45.0);
        let expected = 5.0 + (2050.0 - 100.0) * (45.0 - 5.0) / (4000.0 - 100.0);
        assert!((result - expected).abs() < 0.01);
    }
    
    #[test]
    fn test_convert_zero_point() {
        // 测试零点
        let result = AdConverter::convert(0.0, 0.0, 0.0, 4095.0, 50.0);
        assert!((result - 0.0).abs() < 0.01);
    }
    
    #[test]
    fn test_convert_scale_point() {
        // 测试满量程点
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
