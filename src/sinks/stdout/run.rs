use std::io::Write;

use serde_json::json;

use crate::{pipelining::StageReceiver, Error};

pub fn jsonl_writer_loop(input: StageReceiver, output: &mut impl Write) -> Result<(), Error> {
    loop {
        let evt = input.recv()?;
        let buf = json!(evt).to_string();
        output.write_all(buf.as_bytes())?;
        output.write_all(b"\n")?;
    }
}
