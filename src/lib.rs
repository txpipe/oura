pub mod model;
mod utils;

pub mod filters;
pub mod mapper;
pub mod pipelining;
pub mod sinks;
pub mod sources;

pub type Error = Box<dyn std::error::Error>;
