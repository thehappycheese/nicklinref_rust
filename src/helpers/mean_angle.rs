/// Computes the mean angle from angles in radians
/// (from https://rosettacode.org/wiki/Averages/Mean_angle#Rust)
pub fn mean_angle(angles: Vec<f64>) -> f64 {
    let length: f64 = angles.len() as f64;
    let cos_mean: f64 = angles.iter().fold(0.0, |sum, i| sum + i.cos()) / length;
    let sin_mean: f64 = angles.iter().fold(0.0, |sum, i| sum + i.sin()) / length;
    (sin_mean).atan2(cos_mean)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    #[test]
    fn test_mean_angle_zeros() {
        let avg = mean_angle(vec![0.0, 0.0, 0.0]);
        assert_eq!(avg, 0.0);
    }

    #[test]
    fn test_mean_angle_180s() {
        let avg = mean_angle(vec![
            PI,
            -PI,
            PI,
            PI
        ]);
        assert!(avg.abs()-PI < 0.0001);
    }

    #[test]
    fn test_mean_angle_near_negative_180() {
        let avg = mean_angle(vec![
            f64::to_radians(-170.0),
            f64::to_radians(160.0)
        ]);
        assert!(f64::to_degrees(avg) - 175.0 < 0.0001);
    }
}