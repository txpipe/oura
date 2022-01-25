//! Maps block data into multiple events
//!
//! It uses a visitor pattern to crawl the nested data structures inside a block
//! and writes output events as they are found

mod cip25;
mod collect;
mod crawl;
mod map;
mod prelude;

pub use prelude::*;
