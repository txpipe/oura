use bech32::{FromBase32, ToBase32};
use pallas::crypto::hash::Hash;
use pallas::crypto::hash::Hasher;

pub fn compute_hash(policy_id: &[u8], asset_name: &[u8]) -> Hash<20> {
    let mut hasher = Hasher::<160>::new();
    hasher.input(policy_id);
    hasher.input(asset_name);
    hasher.finalize()
}

#[allow(dead_code)]
pub fn fingerprint(policy_id: &[u8], asset_name: &[u8]) -> anyhow::Result<String> {
    let hash = compute_hash(policy_id, asset_name);
    let base32 = hash.to_base32();
    let x = bech32::encode("asset", base32, bech32::Variant::Bech32)?;
    Ok(x)
}

#[allow(dead_code)]
pub fn read_hash(bech32: &str) -> anyhow::Result<Vec<u8>> {
    let (_, datapart, _) = bech32::decode(bech32)?;
    let x = Vec::<u8>::from_base32(&datapart)?;
    Ok(x)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fingerpint_compute() {
        let quickfp_text = |phex: &str, name: &str| {
            let p = hex::decode(phex).unwrap();
            let n = name.as_bytes().to_vec();
            fingerprint(&p, &n).unwrap()
        };

        let quickfp_hex = |phex: &str, name: &str| {
            let p = hex::decode(phex).unwrap();
            let n = hex::decode(name).unwrap();
            fingerprint(&p, &n).unwrap()
        };

        let fp = quickfp_text(
            "7eae28af2208be856f7a119668ae52a49b73725e326dc16579dcc373",
            "abc1",
        );
        assert_eq!(fp, "asset1hrygjggfkalehpdecfhl52g80940an5rxqct44");

        let fp = quickfp_text(
            "1e349c9bdea19fd6c147626a5260bc44b71635f398b67c59881df209",
            "1231",
        );
        assert_eq!(fp, "asset1tra0mxecpkzgpu8a93jedlqzc9fr9wjwkf2f5y");

        // cip14 official test vector
        let fp = quickfp_text(
            "7eae28af2208be856f7a119668ae52a49b73725e326dc16579dcc373",
            "PATATE", //"504154415445",
        );
        assert_eq!(fp, "asset13n25uv0yaf5kus35fm2k86cqy60z58d9xmde92");

        // cip14 official test vector
        let fp = quickfp_hex(
            "7eae28af2208be856f7a119668ae52a49b73725e326dc16579dcc373",
            "0000000000000000000000000000000000000000000000000000000000000000",
        );
        assert_eq!(fp, "asset1pkpwyknlvul7az0xx8czhl60pyel45rpje4z8w");

        // cip14 official test vector
        let fp = quickfp_hex(
            "1e349c9bdea19fd6c147626a5260bc44b71635f398b67c59881df209",
            "",
        );
        assert_eq!(fp, "asset1uyuxku60yqe57nusqzjx38aan3f2wq6s93f6ea");
    }
}
