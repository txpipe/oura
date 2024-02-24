use pallas::ledger::addresses::{Address, ByronAddress, ShelleyAddress, StakeAddress};
use serde::Deserialize;
use std::str::FromStr;

use super::eval::{MatchOutcome, PatternOf};

#[derive(Deserialize, Clone, Debug, Default)]
pub struct AddressPattern {
    pub byron_address: Option<Vec<u8>>,
    pub payment_part: Option<Vec<u8>>,
    pub delegation_part: Option<Vec<u8>>,
    pub payment_is_script: Option<bool>,
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
                        byron_address: Some(x.to_vec()),
                        ..Default::default()
                    });
                }
                Address::Stake(x) => {
                    return Ok(Self {
                        delegation_part: Some(x.to_vec()),
                        ..Default::default()
                    });
                }
                Address::Shelley(x) => {
                    return Ok(Self {
                        payment_part: Some(x.payment().to_vec()),
                        delegation_part: Some(x.delegation().to_vec()),
                        ..Default::default()
                    });
                }
            }
        }

        Err(anyhow::anyhow!("can't parse address pattern"))
    }
}

#[cfg(test)]
mod tests {
    use super::AddressPattern;

    #[test]
    fn test_vectors() {}
}
