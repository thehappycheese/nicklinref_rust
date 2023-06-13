use serde;
use serde::{Deserialize, Deserializer};

/// replace nan with zero, fail on infinite values
pub fn f32_finite_or_zero<'de, D>(deserializer: D) -> Result<f32, D::Error>
where D:Deserializer<'de>{
    let result = f32::deserialize(deserializer)?;
    match result {
        result if result.is_infinite() => Err(serde::de::Error::custom("must be a finite number")), // malformed input
        result if result.is_nan() => Ok(0.0), // treat nan as missing value and elide with 0
        result => Ok(result)
    }
}

/// Fail on nan or infinite values
pub fn f32_finite_or_fail<'de, D>(deserializer: D) -> Result<f32, D::Error>
where D:Deserializer<'de>{
    let result = f32::deserialize(deserializer)?;
    match result {
        result if result.is_finite() => Ok(result),
        _ => Err(serde::de::Error::custom("must be a finite number")) // malformed input
    }
}