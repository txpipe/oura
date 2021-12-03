use std::{sync::mpsc::Receiver, thread::JoinHandle, time::Duration};

use crate::ports::{Event, EventData};

use crate::utils::throttle::Throttle;

pub type Error = Box<dyn std::error::Error>;

const THROTTLE_MIN_SPAN_MILLIS: u64 = 500;

use crossterm::execute;
use crossterm::style::{Attribute, Colors, StyledContent, PrintStyledContent};
use crossterm::{style, style::Color, style::Stylize, ExecutableCommand};
use std::io::{stdout, Write};

fn type_color(data: &EventData) -> Color {
    match data {
        EventData::Block { body_size, issuer_vkey } => Color::DarkGrey,
        EventData::Transaction { fee, ttl, validity_interval_start } => Color::DarkBlue,
        EventData::TxInput { tx_id, index } => Color::Blue,
        EventData::TxOutput { address, amount } => Color::Blue,
        EventData::OutputAsset { coin, policy, asset, value } => Color::Green,
        EventData::Metadata { key } => Color::Cyan,
        EventData::Mint { policy, asset, quantity } => Color::DarkGreen,
        EventData::NativeScript => Color::White,
        EventData::PlutusScript => Color::White,
        EventData::StakeRegistration => Color::Yellow,
        EventData::StakeDeregistration => Color::DarkYellow,
        EventData::StakeDelegation => Color::Yellow,
        EventData::PoolRegistration => Color::Yellow,
        EventData::PoolRetirement => Color::DarkYellow,
        EventData::GenesisKeyDelegation => Color::Yellow,
        EventData::MoveInstantaneousRewardsCert => Color::Yellow,
    }
}

fn type_prefix(data: &EventData) -> &'static str {
    match data {
        EventData::Block {
            body_size,
            issuer_vkey,
        } => &"BLK",
        EventData::Transaction {
            fee,
            ttl,
            validity_interval_start,
        } => &"TX_",
        EventData::TxInput { tx_id, index } => &"==>",
        EventData::TxOutput { address, amount } => &"<==",
        EventData::OutputAsset {
            coin,
            policy,
            asset,
            value,
        } => &"ASS",
        EventData::Metadata { key } => &"MTD",
        EventData::Mint {
            policy,
            asset,
            quantity,
        } => &"MIN",
        EventData::NativeScript => &"NTS",
        EventData::PlutusScript => &"PLU",
        EventData::StakeRegistration => &"STR",
        EventData::StakeDeregistration => &"STD",
        EventData::StakeDelegation => &"STD",
        EventData::PoolRegistration => &"POO",
        EventData::PoolRetirement => &"POD",
        EventData::GenesisKeyDelegation => &"GEN",
        EventData::MoveInstantaneousRewardsCert => &"MOV",
    }
}

fn styled_type_prefix(event: &Event) -> StyledContent<String> {
    format!("█ {} ", type_prefix(&event.data))
        .stylize()
        .with(type_color(&event.data))
}

fn styled_block_prefix(event: &Event) -> StyledContent<String> {
    format!("┃ block: {} ┃ ", event.context.block_number.unwrap_or_default()).stylize().with(Color::DarkGrey)
}

fn styled_data(event: &Event) -> StyledContent<String> {
    format!("data: {:?}", event.data).stylize().with(Color::White)
}

fn reducer_loop(event_rx: Receiver<Event>) -> Result<(), Error> {
    let mut stdout = stdout();

    let mut throttle = Throttle::new(Duration::from_millis(THROTTLE_MIN_SPAN_MILLIS));

    loop {
        let evt = event_rx.recv()?;
        throttle.wait_turn();
        stdout.execute(PrintStyledContent(styled_type_prefix(&evt)))?;
        stdout.execute(PrintStyledContent(styled_block_prefix(&evt)))?;
        stdout.execute(PrintStyledContent(styled_data(&evt)))?;
        println!();
    }
}

pub fn bootstrap(rx: Receiver<Event>) -> Result<JoinHandle<()>, Error> {
    let handle = std::thread::spawn(move || reducer_loop(rx).unwrap());

    Ok(handle)
}
