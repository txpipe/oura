use std::fmt::{Display, Write};
use std::{sync::mpsc::Receiver, time::Duration};

use crate::framework::{Event, EventData};
use crate::utils::throttle::Throttle;

pub type Error = Box<dyn std::error::Error>;

use crossterm::style::{Print, SetForegroundColor};
use crossterm::{style::Color, style::Stylize, ExecutableCommand};
use std::io::stdout;

fn type_color(data: &EventData) -> Color {
    match data {
        EventData::Block { .. } => Color::Magenta,
        EventData::Transaction { .. } => Color::DarkBlue,
        EventData::TxInput { tx_id: _, index: _ } => Color::Blue,
        EventData::TxOutput { .. } => Color::Blue,
        EventData::OutputAsset { .. } => Color::Green,
        EventData::Metadata { .. } => Color::Yellow,
        EventData::Mint { .. } => Color::DarkGreen,
        EventData::NewNativeScript => Color::White,
        EventData::NewPlutusScript { .. } => Color::White,
        EventData::PlutusScriptRef { .. } => Color::White,
        EventData::StakeRegistration => Color::Magenta,
        EventData::StakeDeregistration => Color::DarkMagenta,
        EventData::StakeDelegation => Color::Magenta,
        EventData::PoolRegistration => Color::Magenta,
        EventData::PoolRetirement => Color::DarkMagenta,
        EventData::GenesisKeyDelegation => Color::Magenta,
        EventData::MoveInstantaneousRewardsCert => Color::Magenta,
    }
}

fn type_prefix(data: &EventData) -> &'static str {
    match data {
        EventData::Block { .. } => "BLOCK",
        EventData::Transaction { .. } => "TX",
        EventData::TxInput { .. } => "STXI",
        EventData::TxOutput { .. } => "UTXO",
        EventData::OutputAsset { .. } => "ASSET",
        EventData::Metadata { .. } => "META",
        EventData::Mint { .. } => "MINT",
        EventData::NewNativeScript => "NATIVE+",
        EventData::NewPlutusScript { .. } => "PLUTUS+",
        EventData::PlutusScriptRef { .. } => "PLUTUS",
        EventData::StakeRegistration => "STAKE+",
        EventData::StakeDeregistration => "STAKE-",
        EventData::StakeDelegation => "STAKE",
        EventData::PoolRegistration => "POOL+",
        EventData::PoolRetirement => "POOL-",
        EventData::GenesisKeyDelegation => "GENESIS",
        EventData::MoveInstantaneousRewardsCert => "MOVE",
    }
}

type MaxWidth = usize;

struct LogLine(Event, MaxWidth);

impl Display for LogLine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let color = type_color(&self.0.data);
        let prefix = type_prefix(&self.0.data);
        let flex_width = self.1 - 40;

        format!(
            "BLOCK:{}>TX:{:-2}",
            self.0.context.block_number.unwrap_or_default(),
            self.0
                .context
                .tx_idx
                .map(|x| format!("{:-2}", x))
                .unwrap_or_else(|| "--".to_string()),
        )
        .stylize()
        .with(Color::DarkGrey)
        .fmt(f)?;

        f.write_char(' ')?;

        format!("█ {:6}", prefix).stylize().with(color).fmt(f)?;
        f.write_char(' ')?;

        {
            let mut debug = format!("{:?}", &self.0.data);
            let max_width = std::cmp::min(debug.len(), flex_width);

            if debug.len() > max_width {
                debug.truncate(max_width - 3);
                debug = format!("{}...", debug);
            }

            debug.stylize().with(Color::White).fmt(f)?;
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
        let line = LogLine(evt, width as usize);
        stdout.execute(SetForegroundColor(Color::White))?;
        stdout.execute(Print(line))?;
    }
}
