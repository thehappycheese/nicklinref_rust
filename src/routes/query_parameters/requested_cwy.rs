use crate::data::esri_serde::Cwy;
use serde;
use serde::de::{Deserialize, Deserializer, Visitor};

use std::fmt;
use std::iter::IntoIterator;

#[derive(Debug, PartialEq, Clone)]
pub enum RequestedCwy {
    L,
    R,
    S,
    LR,
    LS,
    RS,
    LRS,
}

impl Default for RequestedCwy {
    fn default() -> Self {
        RequestedCwy::LRS
    }
}

impl From<u8> for RequestedCwy {
    fn from(item: u8) -> Self {
        match item {
            0b0000_0100 => RequestedCwy::L,
            0b0000_0001 => RequestedCwy::R,
            0b0000_0010 => RequestedCwy::S,
            0b0000_0101 => RequestedCwy::LR,
            0b0000_0110 => RequestedCwy::LS,
            0b0000_0011 => RequestedCwy::RS,
            0b0000_0111 => RequestedCwy::LRS,
            _ => RequestedCwy::LRS,
        }
    }
}

impl IntoIterator for &RequestedCwy {
    type Item = Cwy;
    type IntoIter = std::vec::IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        match self {
            RequestedCwy::L => vec![Cwy::Left].into_iter(),
            RequestedCwy::R => vec![Cwy::Right].into_iter(),
            RequestedCwy::S => vec![Cwy::Single].into_iter(),
            RequestedCwy::LR => vec![Cwy::Left, Cwy::Right].into_iter(),
            RequestedCwy::LS => vec![Cwy::Left, Cwy::Single].into_iter(),
            RequestedCwy::RS => vec![Cwy::Right, Cwy::Single].into_iter(),
            RequestedCwy::LRS => vec![Cwy::Left, Cwy::Right, Cwy::Single].into_iter(),
        }
    }
}

impl PartialEq<Cwy> for RequestedCwy {
    fn eq(&self, other: &Cwy) -> bool {
        match self {
            RequestedCwy::L => other == &Cwy::Left,
            RequestedCwy::R => other == &Cwy::Right,
            RequestedCwy::S => other == &Cwy::Single,
            RequestedCwy::LR => other == &Cwy::Left || other == &Cwy::Right,
            RequestedCwy::LS => other == &Cwy::Left || other == &Cwy::Single,
            RequestedCwy::RS => other == &Cwy::Right || other == &Cwy::Single,
            RequestedCwy::LRS => true,
        }
    }
}

impl<'de> Deserialize<'de> for RequestedCwy {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct VariantVisitor;
        impl<'de> Visitor<'de> for VariantVisitor {
            type Value = RequestedCwy;
            // Format a message stating what data this Visitor expects to receive.
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("expects to receive any of the following values l, r, s, lr, ls, rs, lrs (or any capitalisation thereof)")
            }
            fn visit_borrowed_str<E>(self, s: &'de str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                let mut chars: Vec<char> = s.to_uppercase().chars().collect::<Vec<char>>();
                chars.sort();
                Ok(match &chars.into_iter().collect::<String>()[..] {
                    "L" => RequestedCwy::L,
                    "R" => RequestedCwy::R,
                    "S" => RequestedCwy::S,
                    "LR" => RequestedCwy::LR,
                    "LS" => RequestedCwy::LS,
                    "RS" => RequestedCwy::RS,
                    "LRS" => RequestedCwy::LRS,
                    _ => RequestedCwy::LRS,
                })
            }
        }
        //const VARIANTS: &'static [&'static str] = &["L", "R", "S", "LR", "LS", "RS", "LRS"];
        //deserializer.deserialize_enum("RequestedCwy", VARIANTS, VariantVisitor)
        deserializer.deserialize_string(VariantVisitor)
    }
}
