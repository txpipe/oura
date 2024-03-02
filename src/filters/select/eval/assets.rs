use self::serde_ext::FromBech32;

use super::*;

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct AssetPattern {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fingerprint: Option<FlexBytes>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy: Option<FlexBytes>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<FlexBytes>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub name_text: Option<TextPattern>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub coin: Option<CoinPattern>,
}

impl FromBech32 for AssetPattern {
    fn from_bech32_parts(hrp: &str, content: Vec<u8>) -> Option<Self> {
        match hrp {
            "asset" => Some(Self {
                fingerprint: Some(FlexBytes(content)),
                ..Default::default()
            }),
            _ => None,
        }
    }
}

impl FromStr for AssetPattern {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_bech32(s)
    }
}

impl PatternOf<(&[u8], &Asset)> for AssetPattern {
    fn is_match(&self, subject: (&[u8], &Asset)) -> MatchOutcome {
        let (subject_policy, subject_asset) = subject;

        let a = self.policy.is_match(subject_policy);

        let b = self.name.is_match(subject_asset.name.as_ref());

        let c = self.name_text.is_match(subject_asset.name.as_ref());

        let d = self.coin.is_match(subject_asset.output_coin);

        MatchOutcome::fold_all_of([a, b, c, d].into_iter())
    }
}

impl PatternOf<&Multiasset> for AssetPattern {
    fn is_match(&self, subject: &Multiasset) -> MatchOutcome {
        let policy = subject.policy_id.as_ref();

        let subjects = subject.assets.iter().map(|x| (policy, x));

        self.is_any_match(subjects)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pattern_parse() {
        let expected = AssetPattern {
            fingerprint: Some(
                FlexBytes::from_hex("8cd54e31e4ea696e42344ed563eb00269e2a1da5").unwrap(),
            ),
            ..Default::default()
        };

        let parsed =
            AssetPattern::from_str(&"asset13n25uv0yaf5kus35fm2k86cqy60z58d9xmde92").unwrap();
        assert_eq!(parsed, expected);
    }
}
