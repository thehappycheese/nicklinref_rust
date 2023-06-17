use crate::data::esri_serde::Cwy;
use serde::Deserialize;

#[derive(Debug, PartialEq, Clone, Deserialize)]
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

impl Into<u8> for RequestedCwy {
    fn into(self) -> u8 {
        match self {
            RequestedCwy::L   => 0b0000_0100,
            RequestedCwy::R   => 0b0000_0001,
            RequestedCwy::S   => 0b0000_0010,
            RequestedCwy::LR  => 0b0000_0101,
            RequestedCwy::LS  => 0b0000_0110,
            RequestedCwy::RS  => 0b0000_0011,
            RequestedCwy::LRS => 0b0000_0111,
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