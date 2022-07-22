//! An utility to build fingerprints for Cardano assets CIP14
//! https://cips.cardano.org/cips/cip14/
//! 
use cryptoxide::{digest::Digest, blake2b::Blake2b};
use crate::utils::bech32::{Bech32Config,Bech32Provider};

pub fn blake2b160(data: &[u8]) -> [u8;20] {
    let mut out = [0u8; 20];
    let mut context = Blake2b::new(20);
    context.input(data);
    context.result(&mut out);
    Blake2b::blake2b(&mut out, data, &[]);
    out
}

pub fn cip14_fingerprint(p: &Vec::<u8>, a: &Vec::<u8>) -> Option<String> {
    let data = [&p[..],&a[..]].concat();
    let hash = blake2b160(&data);
    let bech32_provider = Bech32Provider::new(Bech32Config::for_cip14());
    let fingerprint = bech32_provider.encode_cip14(hash.as_slice());
    log::debug!("CIP14 Fingerprint: {:?}",fingerprint);
    fingerprint
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cip14_ok_1() {
        let policy = hex::decode("bb3ce45d5272654e58ad076f114d8f683ae4553e3c9455b18facfea1").unwrap();
        let assetname =hex::decode("4261627943726f63202332323237").unwrap();

        let bech32 = cip14_fingerprint(&policy,&assetname).unwrap();
        log::debug!("{}",bech32);
        assert_eq!(bech32, "asset1et8j5whwuqrxvdyxfh4grmmrx4exeg4juzx88z");
    }

    #[test]
    fn cip14_ok_2() {
        let policy = hex::decode("7eae28af2208be856f7a119668ae52a49b73725e326dc16579dcc373").unwrap();
        let assetname =hex::decode("").unwrap();

        let bech32 = cip14_fingerprint(&policy,&assetname).unwrap();
        log::debug!("{}",bech32);
        assert_eq!(bech32, "asset1rjklcrnsdzqp65wjgrg55sy9723kw09mlgvlc3");
    }

    #[test]
    fn cip14_ok_3() {
        let policy = hex::decode("7eae28af2208be856f7a119668ae52a49b73725e326dc16579dcc37e").unwrap();
        let assetname =hex::decode("").unwrap();

        let bech32 = cip14_fingerprint(&policy,&assetname).unwrap();
        log::debug!("{}",bech32);
        assert_eq!(bech32, "asset1nl0puwxmhas8fawxp8nx4e2q3wekg969n2auw3");
    }

    #[test]
    fn cip14_ok_4() {
        let policy = hex::decode("1e349c9bdea19fd6c147626a5260bc44b71635f398b67c59881df209").unwrap();
        let assetname =hex::decode("").unwrap();

        let bech32 = cip14_fingerprint(&policy,&assetname).unwrap();
        log::debug!("{}",bech32);
        assert_eq!(bech32, "asset1uyuxku60yqe57nusqzjx38aan3f2wq6s93f6ea");
    }

    #[test]
    fn cip14_ok_5() {
        let policy = hex::decode("7eae28af2208be856f7a119668ae52a49b73725e326dc16579dcc373").unwrap();
        let assetname =hex::decode("504154415445").unwrap();

        let bech32 = cip14_fingerprint(&policy,&assetname).unwrap();
        log::debug!("{}",bech32);
        assert_eq!(bech32, "asset13n25uv0yaf5kus35fm2k86cqy60z58d9xmde92");
    }

    #[test]
    fn cip14_ok_6() {
        let policy = hex::decode("1e349c9bdea19fd6c147626a5260bc44b71635f398b67c59881df209").unwrap();
        let assetname =hex::decode("504154415445").unwrap();

        let bech32 = cip14_fingerprint(&policy,&assetname).unwrap();
        log::debug!("{}",bech32);
        assert_eq!(bech32, "asset1hv4p5tv2a837mzqrst04d0dcptdjmluqvdx9k3");
    }

    #[test]
    fn cip14_ok_7() {
        let policy = hex::decode("1e349c9bdea19fd6c147626a5260bc44b71635f398b67c59881df209").unwrap();
        let assetname =hex::decode("7eae28af2208be856f7a119668ae52a49b73725e326dc16579dcc373").unwrap();

        let bech32 = cip14_fingerprint(&policy,&assetname).unwrap();
        log::debug!("{}",bech32);
        assert_eq!(bech32, "asset1aqrdypg669jgazruv5ah07nuyqe0wxjhe2el6f");
    }

    #[test]
    fn cip14_ok_8() {
        let policy = hex::decode("7eae28af2208be856f7a119668ae52a49b73725e326dc16579dcc373").unwrap();
        let assetname =hex::decode("1e349c9bdea19fd6c147626a5260bc44b71635f398b67c59881df209").unwrap();

        let bech32 = cip14_fingerprint(&policy,&assetname).unwrap();
        log::debug!("{}",bech32);
        assert_eq!(bech32, "asset17jd78wukhtrnmjh3fngzasxm8rck0l2r4hhyyt");
    }

    #[test]
    fn cip14_ok_9() {
        let policy = hex::decode("7eae28af2208be856f7a119668ae52a49b73725e326dc16579dcc373").unwrap();
        let assetname =hex::decode("0000000000000000000000000000000000000000000000000000000000000000").unwrap();

        let bech32 = cip14_fingerprint(&policy,&assetname).unwrap();
        log::debug!("{}",bech32);
        assert_eq!(bech32, "asset1pkpwyknlvul7az0xx8czhl60pyel45rpje4z8w");

    }
    
}
