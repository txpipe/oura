mod framework;
mod utils;

pub mod pipelining;
pub mod filters;
pub mod mapper;
pub mod sinks;
pub mod sources;


pub type Error = Box<dyn std::error::Error>;
