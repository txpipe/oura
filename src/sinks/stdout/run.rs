use std::{io::Write, sync::Arc};

use serde_json::json;

use crate::{pipelining::StageReceiver, utils::Utils, Error};

pub fn jsonl_writer_loop(
    input: StageReceiver,
    output: &mut impl Write,
    utils: Arc<Utils>,
) -> Result<(), Error> {
    loop {
        let evt = input.recv()?;

        // notify pipeline about the progress
        utils.track_sink_progress(&evt);

        let buf = json!(evt).to_string();
        output.write_all(buf.as_bytes())?;
        output.write_all(b"\n")?;
    }
}
