/// Computes the mean angle from angles in radians
/// (from https://rosettacode.org/wiki/Averages/Mean_angle#Rust)
pub fn mean_angle(angles: Vec<f64>) -> f64 {
    let length: f64 = angles.len() as f64;
    let cos_mean: f64 = angles.iter().fold(0.0, |sum, i| sum + i.cos()) / length;
    let sin_mean: f64 = angles.iter().fold(0.0, |sum, i| sum + i.sin()) / length;
    (sin_mean).atan2(cos_mean)
}