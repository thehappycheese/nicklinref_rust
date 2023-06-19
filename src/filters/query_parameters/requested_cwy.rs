use crate::data::cached::Cwy;
use serde::Deserialize;

#[derive(Debug, PartialEq, Clone, Deserialize)]
pub enum RequestedCwy {
    L,
    R,
    S,
    #[serde(alias = "RL")] 
    LR,
    #[serde(alias = "SL")] 
    LS,
    #[serde(alias = "SR")] 
    RS,
    #[serde(alias = "RLS")] 
    #[serde(alias = "RSL")] 
    #[serde(alias = "SLR")]
    #[serde(alias = "SRL")]
    #[serde(alias = "LSR")]
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

/// This is actually a 'membership' test rather than an 'equals' test.
/// On reflection it might be better to convert this to a named function
/// instead of extending the equality operator. I think I did it to save a line 
/// of code in an iterator somewhere.
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

#[cfg(test)]
mod test {
    use super::*;
    use serde_json;
    
    #[test]
    /// A sanity check to confirm serde is deserializing all variants correctly
    /// The ability to use any permutation is an undocumented feature of the api
    fn requested_cwy_deserialize(){
        
        macro_rules! test_parse {
            ($serial:expr, $variant:ident) => {
                let res: RequestedCwy = serde_json::from_str(&format!("\"{}\"", $serial)).unwrap();
                assert_eq!(res, RequestedCwy::$variant);
            };
        }

        test_parse!("L", L);
        test_parse!("R", R);
        test_parse!("S", S);

        test_parse!("LR", LR);
        test_parse!("RL", LR);
        
        test_parse!("LS", LS);
        test_parse!("SL", LS);

        test_parse!("RS", RS);
        test_parse!("SR", RS);

        test_parse!("LRS", LRS);
        test_parse!("LSR", LRS);
        test_parse!("RLS", LRS);
        test_parse!("RSL", LRS);
        test_parse!("SLR", LRS);
        test_parse!("SRL", LRS);
        
    }
}