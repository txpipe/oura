use std::fmt::{Display, Write};
use std::{sync::mpsc::Receiver, time::Duration};

use crate::framework::{Event, EventData};
use crate::utils::throttle::Throttle;

pub type Error = Box<dyn std::error::Error>;

use crossterm::style::{Print, SetForegroundColor, StyledContent, ContentStyle};
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
            EventData::OutputAsset { policy, asset, amount } => LogLine {
                prefix: "ASSET",
                color: Color::Green,
                content: format!("{{ policy: {}, asset: {}, amount: {} }}", policy, asset, amount),
                source,
                max_width,
            },
            EventData::Metadata { key, subkey, value } => LogLine {
                prefix: "META",
                color: Color::Yellow,
                content: format!("{{ key: {}, sub key: {:?}, value: {:?} }}", key, subkey, value),
                source,
                max_width,
            },
            EventData::Mint { policy, asset, quantity } => LogLine {
                prefix: "MINT",
                color: Color::DarkGreen,
                content: format!("{{ policy: {}, asset: {}, quantity: {} }}", policy, asset, quantity),
                source,
                max_width,
            },
            EventData::NewNativeScript => LogLine {
                prefix: "NATIVE+",
                color: Color::White,
                content: format!("{{ ... }}"),
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
            EventData::StakeRegistration => LogLine {
                prefix: "STAKE+",
                color: Color::Magenta,
                content: format!("{{ ... }}"),
                source,
                max_width,
            },
            EventData::StakeDeregistration => LogLine {
                prefix: "STAKE-",
                color: Color::DarkMagenta,
                content: format!("{{ ... }}"),
                source,
                max_width,
            },
            EventData::StakeDelegation => LogLine {
                prefix: "DELE",
                color: Color::Magenta,
                content: format!("{{ ... }}"),
                source,
                max_width,
            },
            EventData::PoolRegistration => LogLine {
                prefix: "POOL+",
                color: Color::Magenta,
                content: format!("{{ ... }}"),
                source,
                max_width,
            },
            EventData::PoolRetirement => LogLine {
                prefix: "POOL-",
                color: Color::DarkMagenta,
                content: format!("{{ ... }}"),
                source,
                max_width,
            },
            EventData::GenesisKeyDelegation => LogLine {
                prefix: "GENESIS",
                color: Color::Magenta,
                content: format!("{{ ... }}"),
                source,
                max_width,
            },
            EventData::MoveInstantaneousRewardsCert => LogLine {
                prefix: "MOVE",
                color: Color::Magenta,
                content: format!("{{ ... }}"),
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
            "BLOCK:{} █ TX:{:-2}",
            self.source.context.block_number.unwrap_or_default(),
            self.source
                .context
                .tx_idx
                .map(|x| format!("{:-2}", x))
                .unwrap_or_else(|| "--".to_string()),
        )
        .stylize()
        .with(Color::DarkGrey)
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
                    let partial = &self.content[..max_width-3];
                    partial.with(Color::White).fmt(f)?;
                    f.write_str("...")?;
                }
                _ => {                   
                    let full = &self.content[..]; 
                    full.with(Color::White).fmt(f)?;
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
