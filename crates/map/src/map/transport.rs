use serde::Deserialize;
use serde::Serialize;
use std::fmt;

/// Transport mode of an edge (a travel route between two stations).
///
/// Maps to the Scotland Yard ticket types: taxis and buses are public
/// for everyone, river and underground routes are Fugitive's
/// secret-ticket territory (black ticket / sewer pass).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Transport {
    /// Short street connection; paid with a taxi ticket.
    Taxi,
    /// Mid-range bus route; paid with a bus ticket.
    Bus,
    /// Long-range metro connection; paid with a metro ticket.
    Metro,
    /// Seine river route (Batobus); Fugitive's black ticket.
    Batobus,
    /// Underground network (Paris: Égouts, Berlin: Unterwelten) -
    /// the fugitive's secret tunnels.
    Underground,
}

impl Transport {
    /// Every transport kind.
    pub const ALL: [Self; 5] = [Self::Taxi, Self::Bus, Self::Metro, Self::Batobus, Self::Underground];
}

impl fmt::Display for Transport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Taxi => "taxi",
            Self::Bus => "bus",
            Self::Metro => "metro",
            Self::Batobus => "batobus",
            Self::Underground => "underground",
        };
        f.write_str(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toml_roundtrip() {
        #[derive(Debug, PartialEq, Serialize, Deserialize)]
        struct Wrapper {
            /// Transport under test.
            t: Transport,
        }
        for transport in Transport::ALL {
            let encoded = toml::to_string(&Wrapper { t: transport }).unwrap_or_default();
            assert_eq!(toml::from_str::<Wrapper>(&encoded).ok(), Some(Wrapper { t: transport }));
        }
    }
}
