use super::*;

///! A friendly facade to simplify access to common utils procedures

impl Utils {
    // To be used by source stages to access the cursor, if any
    pub fn get_cursor_if_any(&self) -> Option<cursor::PointArg> {
        match &self.cursor {
            Some(provider) => provider.get_cursor(),
            _ => None,
        }
    }

    /// To be used by sink stages to track progress
    pub fn track_source_progress(&self, event: &Event) {
        #[cfg(feature = "metrics")]
        self.metrics.on_source_event(event);
    }

    /// To be used by sink stages to track progress
    pub fn track_sink_progress(&self, event: &Event) {
        let point = match (event.context.slot, &event.context.block_hash) {
            (Some(slot), Some(hash)) => cursor::PointArg(slot, hash.to_owned()),
            _ => return,
        };

        if let Some(cursor) = &self.cursor {
            cursor.set_cursor(point).ok_or_warn("failed to set cursor")
        }

        #[cfg(feature = "metrics")]
        self.metrics.on_sink_event(event);
    }

    #[cfg(feature = "metrics")]
    pub fn track_chain_tip(&self, tip: impl Into<metrics::Tip>) {
        //self.metrics.update_chain_state(metrics::ChainState {
        //    tip: Some(tip.into()),
        //    ..Default::default()
        //})
    }
}
