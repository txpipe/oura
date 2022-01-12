pub mod terminal;

pub mod webhook;

#[cfg(feature = "tuisink")]
pub mod tui;

#[cfg(feature = "kafkasink")]
pub mod kafka;

#[cfg(feature = "elasticsink")]
pub mod elastic;
