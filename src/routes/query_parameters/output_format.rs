use serde;
use serde::de::{Deserialize, Deserializer, Visitor};
use std::fmt;

#[derive(Debug)]
pub enum OutputFormat {
    GEOJSON,
    WKT,
    JSON,
    LATLON,
    LATLONDIR,
}

impl Default for OutputFormat {
    fn default() -> Self {
        OutputFormat::GEOJSON
    }
}

impl<'de> Deserialize<'de> for OutputFormat {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct VariantVisitor;

        impl<'de> Visitor<'de> for VariantVisitor {
            type Value = OutputFormat;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("expects to receive any of the following values l, r, s, lr, ls, rs, lrs (or any capitalization thereof)")
            }

            fn visit_borrowed_str<E>(self, s: &'de str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                let chars = s.to_uppercase().chars().collect::<String>();
                Ok(match chars.as_str() {
                    "GEOJSON" => OutputFormat::GEOJSON,
                    "WKT" => OutputFormat::WKT,
                    "JSON" => OutputFormat::JSON,
                    "LATLON" => OutputFormat::LATLON,
                    "LATLONDIR" => OutputFormat::LATLONDIR,
                    _ => OutputFormat::GEOJSON,
                })
            }
        }

        deserializer.deserialize_str(VariantVisitor)
    }
}
