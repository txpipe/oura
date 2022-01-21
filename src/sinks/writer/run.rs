use std::io::Write;

use serde_json::json;

use crate::{framework::Event, pipelining::StageReceiver, Error};

use super::setup::OutputFormat;

fn write(event: Event, format: &OutputFormat, output: &mut impl Write) -> Result<(), Error> {
    match format {
        OutputFormat::JSONL => {
            let buf = json!(event).to_string();
            output.write_all(buf.as_bytes())?;
            output.write(b"\n")?;
            Ok(())
        }
    }
}

pub fn consumer_loop(format: OutputFormat, input: StageReceiver, output: &mut impl Write) -> Result<(), Error> {
    loop {
        let evt = input.recv()?;
        write(evt, &format, output)?;
    }
}
