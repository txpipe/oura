use std::fmt::{Display, Write};

use crossterm::style::{Attribute, Color, Stylize};

use crate::{
    model::{
        BlockRecord, CIP15AssetRecord, CIP25AssetRecord, Event, EventData, MetadataRecord,
        MintRecord, NativeWitnessRecord, OutputAssetRecord, PlutusDatumRecord,
        PlutusRedeemerRecord, PlutusWitnessRecord, TransactionRecord, TxInputRecord,
        TxOutputRecord, VKeyWitnessRecord,
    },
    utils::Utils,
};

pub struct LogLine {
    prefix: &'static str,
    color: Color,
    source: Event,
    content: String,
    max_width: usize,
}

impl LogLine {
    pub fn new(source: Event, max_width: usize, utils: &Utils) -> LogLine {
        match &source.data {
            EventData::Block(BlockRecord {
                era,
                body_size,
                issuer_vkey,
                tx_count,
                slot,
                hash,
                number,
                ..
            }) => {
                LogLine {
                    prefix: "BLOCK",
                    color: Color::Magenta,
                    content: format!(
                    "{{ era: {:?}, slot: {}, hash: {}, number: {}, body size: {}, tx_count: {}, issuer vkey: {}, timestamp: {} }}",
                    era,
                    slot,
                    hash,
                    number,
                    body_size,
                    tx_count,
                    issuer_vkey,
                    source.context.timestamp.unwrap_or_default(),
                ),
                    source,
                    max_width,
                }
            }
            EventData::BlockEnd(BlockRecord {
                slot,
                hash,
                number,
                ..
            }) => {
                LogLine {
                    prefix: "ENDBLK",
                    color: Color::DarkMagenta,
                    content: format!(
                    "{{ slot: {}, hash: {}, number: {} }}",
                    slot,
                    hash,
                    number,
                ),
                    source,
                    max_width,
                }
            }
            EventData::Transaction(TransactionRecord {
                total_output,
                fee,
                ttl,
                hash,
                ..
            }) => LogLine {
                prefix: "TX",
                color: Color::DarkBlue,
                content: format!(
                    "{{ total_output: {}, fee: {}, hash: {}, ttl: {:?} }}",
                    total_output, fee, hash, ttl
                ),
                source,
                max_width,
            },
            EventData::TransactionEnd(TransactionRecord { hash, .. }) => LogLine {
                prefix: "ENDTX",
                color: Color::DarkBlue,
                content: format!(
                    "{{ hash: {} }}",
                    hash
                ),
                source,
                max_width,
            },
            EventData::TxInput(TxInputRecord { tx_id, index }) => LogLine {
                prefix: "STXI",
                color: Color::Blue,
                content: format!("{{ tx_id: {}, index: {} }}", tx_id, index),
                source,
                max_width,
            },
            EventData::TxOutput(TxOutputRecord {
                address, amount, ..
            }) => LogLine {
                prefix: "UTXO",
                color: Color::Blue,
                content: format!("{{ to: {}, amount: {} }}", address, amount),
                source,
                max_width,
            },
            EventData::OutputAsset(OutputAssetRecord {
                policy,
                asset,
                asset_ascii,
                ..
            }) if policy == &utils.well_known.adahandle_policy => LogLine {
                prefix: "$HNDL",
                color: Color::DarkGreen,
                content: format!(
                    "{{ {} => {} }}",
                    asset_ascii.as_deref().unwrap_or(asset),
                    source.context.output_address.as_deref().unwrap_or_default(),
                ),
                source,
                max_width,
            },
            EventData::OutputAsset(OutputAssetRecord {
                policy,
                asset,
                asset_ascii,
                amount,
                ..
            }) => LogLine {
                prefix: "ASSET",
                color: Color::Green,
                content: format!(
                    "{{ policy: {}, asset: {}, amount: {} }}",
                    policy, asset_ascii.as_deref().unwrap_or(asset), amount
                ),
                source,
                max_width,
            },
            EventData::Metadata(MetadataRecord { label, content }) => LogLine {
                prefix: "META",
                color: Color::Yellow,
                content: format!("{{ label: {}, content: {} }}", label, content),
                source,
                max_width,
            },
            EventData::Mint(MintRecord {
                policy,
                asset,
                quantity,
            }) => LogLine {
                prefix: "MINT",
                color: Color::DarkGreen,
                content: format!(
                    "{{ policy: {}, asset: {}, quantity: {} }}",
                    policy, asset, quantity
                ),
                source,
                max_width,
            },
            EventData::NativeScript { policy_id, script } => LogLine {
                prefix: "NATIVE",
                color: Color::White,
                content: format!("{{ policy: {}, script: {} }}", policy_id, script),
                source,
                max_width,
            },
            EventData::PlutusScript { hash, .. } => LogLine {
                prefix: "PLUTUS",
                color: Color::White,
                content: format!("{{ hash: {} }}", hash),
                source,
                max_width,
            },
            EventData::PlutusDatum(PlutusDatumRecord { datum_hash, .. }) => LogLine {
                prefix: "DATUM",
                color: Color::White,
                content: format!("{{ hash: {} }}", datum_hash),
                source,
                max_width,
            },
            EventData::PlutusRedeemer(PlutusRedeemerRecord { purpose, input_idx, .. }) => LogLine {
                prefix: "REDEEM",
                color: Color::White,
                content: format!("{{ purpose: {}, input: {} }}", purpose, input_idx),
                source,
                max_width,
            },
            EventData::PlutusWitness(PlutusWitnessRecord { script_hash, .. }) => LogLine {
                prefix: "WITNESS",
                color: Color::White,
                content: format!("{{ plutus script: {} }}", script_hash ),
                source,
                max_width,
            },
            EventData::NativeWitness(NativeWitnessRecord { policy_id, .. }) => LogLine {
                prefix: "WITNESS",
                color: Color::White,
                content: format!("{{ native policy: {} }}", policy_id),
                source,
                max_width,
            },
            EventData::VKeyWitness(VKeyWitnessRecord { vkey_hex, .. }) => LogLine {
                prefix: "WITNESS",
                color: Color::White,
                content: format!("{{ vkey: {} }}", vkey_hex),
                source,
                max_width,
            },
            EventData::StakeRegistration { credential } => LogLine {
                prefix: "STAKE+",
                color: Color::Magenta,
                content: format!("{{ credential: {:?} }}", credential),
                source,
                max_width,
            },
            EventData::StakeDeregistration { credential } => LogLine {
                prefix: "STAKE-",
                color: Color::DarkMagenta,
                content: format!("{{ credential: {:?} }}", credential),
                source,
                max_width,
            },
            EventData::StakeDelegation {
                credential,
                pool_hash,
            } => LogLine {
                prefix: "DELE",
                color: Color::Magenta,
                content: format!("{{ credential: {:?}, pool: {} }}", credential, pool_hash),
                source,
                max_width,
            },
            EventData::PoolRegistration {
                operator,
                vrf_keyhash: _,
                pledge,
                cost,
                margin,
                reward_account: _,
                pool_owners: _,
                relays: _,
                pool_metadata,
                pool_metadata_hash: _,
            } => LogLine {
                prefix: "POOL+",
                color: Color::Magenta,
                content: format!(
                    "{{ operator: {}, pledge: {}, cost: {}, margin: {}, metadata: {:?} }}",
                    operator, pledge, cost, margin, pool_metadata
                ),
                source,
                max_width,
            },
            EventData::PoolRetirement { pool, epoch } => LogLine {
                prefix: "POOL-",
                color: Color::DarkMagenta,
                content: format!("{{ pool: {}, epoch: {} }}", pool, epoch),
                source,
                max_width,
            },
            EventData::GenesisKeyDelegation { } => LogLine {
                prefix: "GENESIS",
                color: Color::Magenta,
                content: "{{ ... }}".to_string(),
                source,
                max_width,
            },
            EventData::MoveInstantaneousRewardsCert {
                from_reserves,
                from_treasury,
                to_stake_credentials,
                to_other_pot,
            } => LogLine {
                prefix: "MOVE",
                color: Color::Magenta,
                content: format!(
                    "{{ reserves: {}, treasury: {}, to_credentials: {:?}, to_other_pot: {:?} }}",
                    from_reserves, from_treasury, to_stake_credentials, to_other_pot
                ),
                source,
                max_width,
            },
            EventData::RollBack {
                block_slot,
                block_hash,
            } => LogLine {
                prefix: "RLLBCK",
                color: Color::Red,
                content: format!("{{ slot: {}, hash: {} }}", block_slot, block_hash),
                source,
                max_width,
            },
            EventData::Collateral { tx_id, index } => LogLine {
                prefix: "COLLAT",
                color: Color::Blue,
                content: format!("{{ tx_id: {}, index: {} }}", tx_id, index),
                source,
                max_width,
            },
            EventData::CIP25Asset(CIP25AssetRecord {
                policy,
                asset,
                name,
                image,
                ..
            }) => LogLine {
                prefix: "CIP25",
                color: Color::DarkYellow,
                content: format!(
                    "{{ policy: {}, asset: {}, name: {}, image: {} }}",
                    policy,
                    asset,
                    name.as_deref().unwrap_or("?"),
                    image.as_deref().unwrap_or("?")
                ),
                source,
                max_width,
            },
            EventData::CIP15Asset(CIP15AssetRecord {
                voting_key,
                stake_pub,
                ..
            }) => LogLine {
                prefix: "CIP15",
                color: Color::DarkYellow,
                content: format!(
                    "{{ voting key: {}, stake pub: {} }}",
                    voting_key,
                    stake_pub,
                ),
                source,
                max_width,
            },
        }
    }
}

impl Display for LogLine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let flex_width = self.max_width - 40;

        format!(
            "BLOCK:{:0>7} █ TX:{:0>2}",
            self.source
                .context
                .block_number
                .map(|x| x.to_string())
                .unwrap_or_else(|| "-------".to_string()),
            self.source
                .context
                .tx_idx
                .map(|x| x.to_string())
                .unwrap_or_else(|| "--".to_string()),
        )
        .stylize()
        .with(Color::Grey)
        .attribute(Attribute::Dim)
        .fmt(f)?;

        f.write_char(' ')?;

        format!("█ {:6}", self.prefix)
            .stylize()
            .with(self.color)
            .fmt(f)?;

        f.write_char(' ')?;

        {
            let max_width = std::cmp::min(self.content.len(), flex_width);

            match self.content.len() {
                x if x > max_width => {
                    let partial: String = self.content.chars().take(max_width - 3).collect();
                    partial.with(Color::Grey).fmt(f)?;
                    f.write_str("...")?;
                }
                _ => {
                    let full = &self.content[..];
                    full.with(Color::Grey).fmt(f)?;
                }
            };
        }

        f.write_str("\n")?;
        Ok(())
    }
}
