use pallas::ledger::addresses::{Address, ByronAddress, ShelleyAddress, StakeAddress};
use serde::{de::Visitor, Deserialize, Serialize};
use std::str::FromStr;

use super::{
    eval::{MatchOutcome, PatternOf},
    FlexBytes,
};

#[derive(Serialize, Clone, Debug, Default, PartialEq)]
pub struct AddressPattern {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub byron_address: Option<FlexBytes>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub payment_part: Option<FlexBytes>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub delegation_part: Option<FlexBytes>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub payment_is_script: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub delegation_is_script: Option<bool>,
}

impl PatternOf<&ByronAddress> for AddressPattern {
    fn is_match(&self, subject: &ByronAddress) -> super::eval::MatchOutcome {
        let a = self.byron_address.is_match(&subject.to_vec());

        let b = MatchOutcome::if_false(self.payment_part.is_some());

        let c = MatchOutcome::if_false(self.delegation_part.is_some());

        let d = self.payment_is_script.is_match(false);

        let e = self.delegation_is_script.is_match(false);

        MatchOutcome::fold_all_of([a, b, c, d, e].into_iter())
    }
}

impl PatternOf<&ShelleyAddress> for AddressPattern {
    fn is_match(&self, subject: &ShelleyAddress) -> MatchOutcome {
        let a = MatchOutcome::if_false(self.byron_address.is_some());

        let b = self.payment_part.is_match(&subject.payment().to_vec());

        let c = self
            .delegation_part
            .is_match(&subject.delegation().to_vec());

        let d = self
            .payment_is_script
            .is_match(subject.payment().is_script());

        let e = self
            .delegation_is_script
            .is_match(subject.delegation().is_script());

        MatchOutcome::fold_all_of([a, b, c, d, e].into_iter())
    }
}

impl PatternOf<&StakeAddress> for AddressPattern {
    fn is_match(&self, subject: &StakeAddress) -> MatchOutcome {
        let a = MatchOutcome::if_false(self.byron_address.is_some());

        let b = MatchOutcome::if_false(self.payment_part.is_some());

        let c = self.delegation_part.is_match(&subject.to_vec());

        let d = MatchOutcome::if_false(self.payment_is_script.is_some());

        let e = self.delegation_is_script.is_match(subject.is_script());

        MatchOutcome::fold_all_of([a, b, c, d, e].into_iter())
    }
}

impl PatternOf<&Address> for AddressPattern {
    fn is_match(&self, subject: &Address) -> super::eval::MatchOutcome {
        match subject {
            Address::Byron(addr) => self.is_match(addr),
            Address::Shelley(addr) => self.is_match(addr),
            Address::Stake(addr) => self.is_match(addr),
        }
    }
}

impl PatternOf<&[u8]> for AddressPattern {
    fn is_match(&self, subject: &[u8]) -> MatchOutcome {
        Address::from_bytes(subject)
            .map(|subject| self.is_match(&subject))
            .unwrap_or(MatchOutcome::Uncertain)
    }
}

impl FromStr for AddressPattern {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parse = Address::from_bech32(s);

        if let Ok(addr) = parse {
            match addr {
                Address::Byron(x) => {
                    return Ok(Self {
                        byron_address: Some(x.to_vec().into()),
                        ..Default::default()
                    });
                }
                Address::Stake(x) => {
                    return Ok(Self {
                        delegation_part: Some(x.to_vec().into()),
                        ..Default::default()
                    });
                }
                Address::Shelley(x) => {
                    return Ok(Self {
                        payment_part: Some(x.payment().to_vec().into()),
                        delegation_part: Some(x.delegation().to_vec().into()),
                        ..Default::default()
                    });
                }
            }
        }

        Err(anyhow::anyhow!("can't parse address pattern"))
    }
}

struct AddressPatternVisitor;

impl<'de> Visitor<'de> for AddressPatternVisitor {
    type Value = AddressPattern;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("string or map")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        AddressPattern::from_str(value).map_err(serde::de::Error::custom)
    }

    fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        Deserialize::deserialize(serde::de::value::MapAccessDeserializer::new(map))
    }
}

impl<'de> Deserialize<'de> for AddressPattern {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(AddressPatternVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serde() {
        let pattern: AddressPattern =
         serde_json::from_str("\"addr1qx2fxv2umyhttkxyxp8x0dlpdt3k6cwng5pxj3jhsydzer3n0d3vllmyqwsx5wktcd8cc3sq835lu7drv2xwl2wywfgse35a3x\"").unwrap();

        let expected = AddressPattern {
            payment_part: FlexBytes::from_hex(
                "9493315cd92eb5d8c4304e67b7e16ae36d61d34502694657811a2c8e",
            )
            .unwrap()
            .into(),

            delegation_part: FlexBytes::from_hex(
                "337b62cfff6403a06a3acbc34f8c46003c69fe79a3628cefa9c47251",
            )
            .unwrap()
            .into(),

            ..Default::default()
        };

        assert_eq!(pattern, expected);
    }
}
