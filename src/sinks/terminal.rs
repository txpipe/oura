use std::fmt::{Display, Write};
use std::{sync::mpsc::Receiver, thread::JoinHandle, time::Duration};

use crate::ports::{Event, EventData};

use crate::utils::throttle::Throttle;

pub type Error = Box<dyn std::error::Error>;

const THROTTLE_MIN_SPAN_MILLIS: u64 = 500;

use crossterm::execute;
use crossterm::style::{
    Attribute, Colored, Colors, Print, PrintStyledContent, SetForegroundColor, StyledContent,
};
use crossterm::{style, style::Color, style::Stylize, ExecutableCommand};
use std::io::stdout;

fn type_color(data: &EventData) -> Color {
    match data {
        EventData::Block {
            body_size,
            issuer_vkey,
        } => Color::Magenta,
        EventData::Transaction {
            fee,
            ttl,
            validity_interval_start,
        } => Color::DarkBlue,
        EventData::TxInput { tx_id, index } => Color::Blue,
        EventData::TxOutput { address, amount } => Color::Blue,
        EventData::OutputAsset {
            policy,
            asset,
            amount,
        } => Color::Green,
        EventData::Metadata { key, subkey, value } => Color::Yellow,
        EventData::Mint {
            policy,
            asset,
            quantity,
        } => Color::DarkGreen,
        EventData::NewNativeScript => Color::White,
        EventData::NewPlutusScript { data } => Color::White,
        EventData::PlutusScriptRef { data } => Color::White,
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
        EventData::Block {
            body_size,
            issuer_vkey,
        } => &"BLOCK",
        EventData::Transaction {
            fee,
            ttl,
            validity_interval_start,
        } => &"TX",
        EventData::TxInput { tx_id, index } => &"STXI",
        EventData::TxOutput { address, amount } => &"UTXO",
        EventData::OutputAsset {
            policy,
            asset,
            amount,
        } => &"ASSET",
        EventData::Metadata { key, subkey, value } => &"META",
        EventData::Mint {
            policy,
            asset,
            quantity,
        } => &"MINT",
        EventData::NewNativeScript => &"NATIVE+",
        EventData::NewPlutusScript { data } => &"PLUTUS+",
        EventData::PlutusScriptRef { data } => &"PLUTUS",
        EventData::StakeRegistration => &"STAKE+",
        EventData::StakeDeregistration => &"STAKE-",
        EventData::StakeDelegation => &"STAKE",
        EventData::PoolRegistration => &"POOL+",
        EventData::PoolRetirement => &"POOL-",
        EventData::GenesisKeyDelegation => &"GENESIS",
        EventData::MoveInstantaneousRewardsCert => &"MOVE",
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
                .and_then(|x| Some(format!("{:-2}", x)))
                .unwrap_or("--".to_string()),
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

fn reducer_loop(event_rx: Receiver<Event>) -> Result<(), Error> {
    let mut stdout = stdout();

    let mut throttle = Throttle::new(Duration::from_millis(THROTTLE_MIN_SPAN_MILLIS));

    loop {
        let (width, _) = crossterm::terminal::size()?;
        let evt = event_rx.recv()?;
        throttle.wait_turn();
        let line = LogLine(evt, width as usize);
        stdout.execute(SetForegroundColor(Color::White))?;
        stdout.execute(Print(line))?;
    }
}

pub fn bootstrap(rx: Receiver<Event>) -> Result<JoinHandle<()>, Error> {
    let handle = std::thread::spawn(move || reducer_loop(rx).unwrap());

    Ok(handle)
}
