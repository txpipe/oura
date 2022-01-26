//! Bech32 encoding utils
//!
//! Provides artifacts for encoding different values into bech32 format taking
//! into account the context provided via configuration.

use bech32::{self, ToBase32};
use serde::Deserialize;

use crate::Error;

#[derive(Clone, Deserialize)]
pub struct Bech32Config {
    pub address_hrp: String,
}

impl Default for Bech32Config {
    fn default() -> Self {
        Self {
            address_hrp: "addr".to_string(),
        }
    }
}

#[derive(Clone, Default)]
pub struct Bech32Provider(Bech32Config);

impl Bech32Provider {
    pub fn new(config: Bech32Config) -> Self {
        Bech32Provider(config)
    }

    pub fn encode_address(&self, data: &[u8]) -> Result<String, Error> {
        let enc = bech32::encode(
            &self.0.address_hrp,
            data.to_base32(),
            bech32::Variant::Bech32,
        )?;

        Ok(enc)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn beach32_encodes_ok() {
        let provider = Bech32Provider::new(Bech32Config {
            address_hrp: "addr".to_string(),
        });

        let bytes = hex::decode("01ec6ad5daee9febbe300c6160a36d4daf0c5266ae2fe8245cbb581390629814d8165fd547b6f3f6f55842a5f042bcb113e8e86627bc071f37").unwrap();

        let bech32 = provider.encode_address(bytes.as_slice()).unwrap();

        assert_eq!(bech32, "addr1q8kx44w6a607h03sp3skpgmdfkhsc5nx4ch7sfzuhdvp8yrznq2ds9jl64rmdulk74vy9f0sg27tzylgapnz00q8rumsuhj834");
    }
}
