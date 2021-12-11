use std::fmt::{Display, Write};
use std::{sync::mpsc::Receiver, time::Duration};

use crate::framework::{Event, EventData};
use crate::utils::throttle::Throttle;

pub type Error = Box<dyn std::error::Error>;

use crossterm::style::{Attribute, Print, SetForegroundColor};
use crossterm::{style::Color, style::Stylize, ExecutableCommand};
use std::io::stdout;

struct LogLine {
    prefix: &'static str,
    color: Color,
    source: Event,
    content: String,
    max_width: usize,
}

impl LogLine {
    fn from(source: Event, max_width: usize) -> LogLine {
        match &source.data {
            EventData::Block {
                body_size,
                issuer_vkey,
            } => LogLine {
                prefix: "BLOCK",
                color: Color::Magenta,
                content: format!(
                    "{{ body size: {}, issues vkey: {} }}",
                    body_size, issuer_vkey
                ),
                source,
                max_width,
            },
            EventData::Transaction { fee, hash, ttl, .. } => LogLine {
                prefix: "TX",
                color: Color::DarkBlue,
                content: format!("{{ fee: {}, hash: {:?}, ttl: {:?} }}", fee, hash, ttl),
                source,
                max_width,
            },
            EventData::TxInput { tx_id, index } => LogLine {
                prefix: "STXI",
                color: Color::Blue,
                content: format!("{{ tx id: {}, index: {} }}", tx_id, index),
                source,
                max_width,
            },
            EventData::TxOutput { address, amount } => LogLine {
                prefix: "UTXO",
                color: Color::Blue,
                content: format!("{{ address: {}, amount: {} }}", address, amount),
                source,
                max_width,
            },
            EventData::OutputAsset {
                policy,
                asset,
                amount,
            } => LogLine {
                prefix: "ASSET",
                color: Color::Green,
                content: format!(
                    "{{ policy: {}, asset: {}, amount: {} }}",
                    policy, asset, amount
                ),
                source,
                max_width,
            },
            EventData::Metadata { key, subkey, value } => LogLine {
                prefix: "META",
                color: Color::Yellow,
                content: format!(
                    "{{ key: {}, sub key: {:?}, value: {:?} }}",
                    key, subkey, value
                ),
                source,
                max_width,
            },
            EventData::Mint {
                policy,
                asset,
                quantity,
            } => LogLine {
                prefix: "MINT",
                color: Color::DarkGreen,
                content: format!(
                    "{{ policy: {}, asset: {}, quantity: {} }}",
                    policy, asset, quantity
                ),
                source,
                max_width,
            },
            EventData::NewNativeScript => LogLine {
                prefix: "NATIVE+",
                color: Color::White,
                content: "{{ ... }}".to_string(),
                source,
                max_width,
            },
            EventData::NewPlutusScript { data } => LogLine {
                prefix: "PLUTUS+",
                color: Color::White,
                content: format!("{{ {} }}", data),
                source,
                max_width,
            },
            EventData::PlutusScriptRef { data } => LogLine {
                prefix: "PLUTUS",
                color: Color::White,
                content: format!("{{ {} }}", data),
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
            EventData::GenesisKeyDelegation => LogLine {
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
        .with(Color::DarkGrey)
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
                    let partial = &self.content[..max_width - 3];
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

pub fn reducer_loop(throttle_min_span: Duration, input: Receiver<Event>) -> Result<(), Error> {
    let mut stdout = stdout();

    let mut throttle = Throttle::new(throttle_min_span);

    loop {
        let (width, _) = crossterm::terminal::size()?;
        let evt = input.recv()?;
        throttle.wait_turn();
        let line = LogLine::from(evt, width as usize);
        stdout.execute(SetForegroundColor(Color::White))?;
        stdout.execute(Print(line))?;
    }
}
