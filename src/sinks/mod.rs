pub mod terminal;
pub mod stdout;

#[cfg(feature = "logs")]
pub mod logs;

#[cfg(feature = "webhook")]
pub mod webhook;

#[cfg(feature = "kafkasink")]
pub mod kafka;

#[cfg(feature = "elasticsink")]
pub mod elastic;
