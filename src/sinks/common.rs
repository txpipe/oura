use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub enum ErrorPolicy {
    Continue,
    Exit,
}
