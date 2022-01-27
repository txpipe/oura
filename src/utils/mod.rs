use crate::model::Error;

pub mod throttle;

pub(crate) mod bech32;
pub(crate) mod time;

pub(crate) trait SwallowResult {
    fn ok_or_warn(self, context: &'static str);
}

impl SwallowResult for Result<(), Error> {
    fn ok_or_warn(self, context: &'static str) {
        match self {
            Ok(_) => (),
            Err(e) => log::warn!("{}: {:?}", context, e),
        }
    }
}
