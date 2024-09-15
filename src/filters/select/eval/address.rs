use pallas::ledger::addresses::{
    Address, ByronAddress, ShelleyAddress, ShelleyDelegationPart, StakeAddress,
};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

use super::*;

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct AddressPattern {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub byron_address: Option<FlexBytes>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payment_part: Option<FlexBytes>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub delegation_part: Option<FlexBytes>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payment_is_script: Option<bool>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub delegation_is_script: Option<bool>,
}

impl PatternOf<&ByronAddress> for AddressPattern {
    fn is_match(&self, subject: &ByronAddress) -> MatchOutcome {
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
    fn is_match(&self, subject: &Address) -> MatchOutcome {
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

impl From<Address> for AddressPattern {
    fn from(value: Address) -> Self {
        match value {
            Address::Byron(x) => Self {
                byron_address: Some(x.to_vec().into()),
                ..Default::default()
            },
            Address::Stake(x) => Self {
                delegation_part: Some(x.to_vec().into()),
                ..Default::default()
            },
            Address::Shelley(x) => Self {
                payment_part: Some(x.payment().to_vec().into()),
                delegation_part: match x.delegation() {
                    ShelleyDelegationPart::Key(x) => Some(x.to_vec().into()),
                    ShelleyDelegationPart::Script(x) => Some(x.to_vec().into()),
                    _ => None,
                },
                ..Default::default()
            },
        }
    }
}

impl FromBech32 for AddressPattern {
    fn from_bech32_parts(hrp: &str, content: Vec<u8>) -> Option<Self> {
        match hrp {
            "addr" | "addr_test" | "stake" => Address::from_bytes(&content).ok().map(From::from),
            // TODO: add vk prefixes
            _ => None,
        }
    }
}

impl FromStr for AddressPattern {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        AddressPattern::from_bech32(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_full_address_pattern() {
        let pattern = AddressPattern::from_str("addr1qx2fxv2umyhttkxyxp8x0dlpdt3k6cwng5pxj3jhsydzer3n0d3vllmyqwsx5wktcd8cc3sq835lu7drv2xwl2wywfgse35a3x").unwrap();

        dbg!(&pattern);

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

    #[test]
    fn parse_payment_part_pattern() {
        let pattern =
            AddressPattern::from_str("addr1w8ax5k9mutg07p2ngscu3chsauktmstq92z9de938j8nqacprc9mw")
                .unwrap();

        let expected = AddressPattern {
            payment_part: FlexBytes::from_hex(
                "fa6a58bbe2d0ff05534431c8e2f0ef2cbdc1602a8456e4b13c8f3077",
            )
            .unwrap()
            .into(),

            delegation_part: None,

            ..Default::default()
        };

        assert_eq!(pattern, expected);
    }

    #[test]
    fn address_match() {
        let pattern = |addr: &str| Pattern::from(AddressPattern::from_str(addr).unwrap());

        let possitives = testing::find_positive_test_vectors(pattern(
            "addr1qx2fxv2umyhttkxyxp8x0dlpdt3k6cwng5pxj3jhsydzer3n0d3vllmyqwsx5wktcd8cc3sq835lu7drv2xwl2wywfgse35a3x"
        ));
        assert_eq!(possitives, vec![1, 3]);

        let possitives = testing::find_positive_test_vectors(pattern(
            "addr1vx2fxv2umyhttkxyxp8x0dlpdt3k6cwng5pxj3jhsydzers66hrl8",
        ));
        assert_eq!(possitives, vec![1, 2, 3]);
    }
}
