use super::*;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MetadatumPattern {
    Text(TextPattern),
    Int(NumericPattern<i64>),
    // TODO: bytes, array, map
}

impl PatternOf<&Metadatum> for MetadatumPattern {
    fn is_match(&self, subject: &Metadatum) -> MatchOutcome {
        match self {
            MetadatumPattern::Text(x) => x.is_match(subject),
            MetadatumPattern::Int(_) => todo!(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct MetadataPattern {
    #[serde(skip_serializing_if = "Option::is_none")]
    label: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    value: Option<MetadatumPattern>,
}

use regex::Regex;

impl FromStr for MetadataPattern {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let re = Regex::new(r"#(\d+)").unwrap();

        if let Some(caps) = re.captures(s) {
            if caps.len() == 2 {
                let label = caps[1].parse()?;

                return Ok(Self {
                    label: Some(label),
                    ..Default::default()
                });
            }
        }

        anyhow::bail!("can't parse string as metadata pattern (expected #<u64>)");
    }
}

impl PatternOf<&Metadata> for MetadataPattern {
    fn is_match(&self, subject: &Metadata) -> MatchOutcome {
        let a = self.label.is_match(subject.label);

        let b = self.value.is_any_match(subject.value.iter());

        MatchOutcome::fold_all_of([a, b].into_iter())
    }
}

impl PatternOf<&AuxData> for MetadataPattern {
    fn is_match(&self, subject: &AuxData) -> MatchOutcome {
        self.is_any_match(subject.metadata.iter())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pattern_parse() {
        let expected = MetadataPattern {
            label: Some(127),
            ..Default::default()
        };

        let parsed = MetadataPattern::from_str("#127").unwrap();
        assert_eq!(parsed, expected);
    }

    #[test]
    fn label_match() {
        let pattern = |label: u64| {
            Pattern::Metadata(
                MetadataPattern {
                    label: Some(label),
                    ..Default::default()
                }
                .into(),
            )
        };

        let positives = testing::find_positive_test_vectors(pattern(127));
        assert_eq!(positives, vec![1, 2]);

        let positives = testing::find_positive_test_vectors(pattern(9980));
        assert_eq!(positives, vec![1, 3]);

        let positives = testing::find_positive_test_vectors(pattern(66666));
        assert_eq!(positives, Vec::<usize>::new());
    }

    #[test]
    fn fingerprint_match() {
        let pattern = |fp: &str| Pattern::from(AssetPattern::from_str(fp).unwrap());

        let positives = testing::find_positive_test_vectors(pattern(
            "asset1hrygjggfkalehpdecfhl52g80940an5rxqct44",
        ));
        assert_eq!(positives, [1, 2]);

        let positives = testing::find_positive_test_vectors(pattern(
            "asset1tra0mxecpkzgpu8a93jedlqzc9fr9wjwkf2f5y",
        ));
        assert_eq!(positives, [1, 3]);

        let positives = testing::find_positive_test_vectors(pattern(
            "asset13n25uv0yaf5kus35fm2k86cqy60z58d9xmde92",
        ));
        assert_eq!(positives, Vec::<usize>::new());
    }

    #[test]
    fn regex_text_value_match() {
        use regex::Regex;

        let pattern = MetadataPattern {
            label: Some(674),
            value: Some(MetadatumPattern::Text(TextPattern::Regex(
                Regex::new(r"testing regex").unwrap(),
            ))),
        };

        assert!(pattern.label.is_some());
        assert!(pattern.value.is_some());

        if let Some(MetadatumPattern::Text(TextPattern::Regex(regex))) = &pattern.value {
            assert!(regex.is_match("testing regex"));
            assert!(regex.is_match("this contains testing regex inside"));
            assert!(!regex.is_match("no match here"));
        } else {
            panic!("Expected Text(Regex) pattern");
        }
    }

    /// Tests regex pattern matching against different metadatum types.
    #[test]
    fn regex_text_value_matches_metadatum() {
        use pallas::interop::utxorpc::spec::cardano::metadatum;
        use regex::Regex;

        let text_pattern = TextPattern::Regex(Regex::new(r"Hello World").unwrap());

        let text_metadatum = Metadatum {
            metadatum: metadatum::Metadatum::Text("Hello World".to_string()).into(),
        };
        assert_eq!(
            text_pattern.is_match(&text_metadatum),
            MatchOutcome::Positive
        );

        let no_match = Metadatum {
            metadatum: metadatum::Metadatum::Text("Goodbye".to_string()).into(),
        };
        assert_eq!(text_pattern.is_match(&no_match), MatchOutcome::Negative);

        let int_metadatum = Metadatum {
            metadatum: metadatum::Metadatum::Int(42).into(),
        };
        assert_eq!(
            text_pattern.is_match(&int_metadatum),
            MatchOutcome::Negative
        );

        let bytes_metadatum = Metadatum {
            metadatum: metadatum::Metadatum::Bytes(vec![0xFF, 0xFE, 0xFD].into()).into(),
        };
        assert_eq!(
            text_pattern.is_match(&bytes_metadatum),
            MatchOutcome::Negative
        );
    }

}
