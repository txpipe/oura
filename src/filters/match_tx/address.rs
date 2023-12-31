#[derive(Deserialize, Clone, Debug)]
pub enum AddressPatternValue {
    ExactHex(String),
    ExactBech32(String),
    PaymentHex(String),
    PaymentBech32(String),
    StakeHex(String),
    StakeBech32(String),
}

#[derive(Deserialize, Clone, Debug)]
pub struct AddressPattern {
    pub value: AddressPatternValue,
    pub is_script: Option<bool>,
}

impl AddressPattern {
    fn address_match(&self, address: &Address) -> Result<bool, WorkerError> {
        match address {
            Address::Byron(addr) => match &self.value {
                AddressPatternValue::ExactHex(exact_hex) => Ok(addr.to_hex().eq(exact_hex)),
                AddressPatternValue::PaymentHex(payment_hex) => Ok(addr.to_hex().eq(payment_hex)),
                _ => Ok(false),
            },
            Address::Shelley(addr) => match &self.value {
                AddressPatternValue::ExactHex(exact_hex) => Ok(addr.to_hex().eq(exact_hex)),
                AddressPatternValue::ExactBech32(exact_bech32) => {
                    Ok(addr.to_bech32().or_panic()?.eq(exact_bech32))
                }
                AddressPatternValue::PaymentHex(payment_hex) => {
                    Ok(addr.payment().to_hex().eq(payment_hex))
                }
                AddressPatternValue::PaymentBech32(payment_bech32) => {
                    Ok(addr.payment().to_bech32().or_panic()?.eq(payment_bech32))
                }
                AddressPatternValue::StakeHex(stake_hex) => {
                    if addr.delegation().as_hash().is_none() {
                        return Ok(false);
                    }

                    let stake_address: StakeAddress = addr.clone().try_into().or_panic()?;
                    Ok(stake_address.to_hex().eq(stake_hex))
                }
                AddressPatternValue::StakeBech32(stake_bech32) => {
                    if addr.delegation().as_hash().is_none() {
                        return Ok(false);
                    }

                    let stake_address: StakeAddress = addr.clone().try_into().or_panic()?;
                    Ok(stake_address.to_bech32().or_panic()?.eq(stake_bech32))
                }
            },
            Address::Stake(stake_address) => match &self.value {
                AddressPatternValue::StakeHex(stake_hex) => {
                    Ok(stake_address.to_hex().eq(stake_hex))
                }
                AddressPatternValue::StakeBech32(stake_bech32) => {
                    Ok(stake_address.to_bech32().or_panic()?.eq(stake_bech32))
                }
                _ => Ok(false),
            },
        }
    }
}
