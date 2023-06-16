mod with_shared_data;
pub use with_shared_data::with_shared_data;

mod error_with_static_message;
pub use error_with_static_message::ErrorWithStaticMessage;

mod unit_conversion;
pub use unit_conversion::convert_metres_to_degrees;

mod mean_angle;
pub use mean_angle::mean_angle;

pub mod serde_helpers;