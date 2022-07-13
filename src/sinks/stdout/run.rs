use std::{io::Write, sync::Arc};

use serde_json::json;

use crate::{pipelining::StageReceiver, utils::Utils, Error};

pub fn jsonl_writer_loop(
    input: StageReceiver,
    output: &mut impl Write,
    utils: Arc<Utils>,
) -> Result<(), Error> {
    for evt in input.iter() {
        let buf = json!(evt).to_string();

        let result = output
            .write_all(buf.as_bytes())
            .and_then(|_| output.write_all(b"\n"));

        match result {
            Ok(_) => {
                // notify pipeline about the progress
                utils.track_sink_progress(&evt);
            }
            Err(err) => return Err(Box::new(err)),
        }
    }

    Ok(())
}
