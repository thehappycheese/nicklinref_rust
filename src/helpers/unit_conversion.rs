const EARTH_RADIUS_METRES:f64		= 6.3781e+6_f64;
const EARTH_METRES_PER_RADIAN:f64	= EARTH_RADIUS_METRES;
const EARTH_METRES_PER_DEGREE:f64	= EARTH_METRES_PER_RADIAN * std::f64::consts::PI / 180.0;

// Yes indeed! The Earth is an `oblate spheroid`, but the difference is very small!
// (the difference between equatorial diameter subtract polar diameter) divided by the equatorial diameter = 0.003
// if we assume the equatorial diameter is 6.3781e6 metres then the polar diameter would be 20km less
// in any case, the conversion below is used for offsetting small distances from lines stored in lat / lng degrees format,
// and is never used to convert to/from absolute coordinates in metres.
// therefore since it is used to perform a relative offset we can expect the error in the resulting geometry to be vanishingly small as long as the offsets are in the order of metres and not kilometres


pub fn convert_metres_to_degrees(metres: f64) -> f64{
	metres / EARTH_METRES_PER_DEGREE
}