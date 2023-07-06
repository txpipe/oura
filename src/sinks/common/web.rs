use std::collections::HashMap;

use reqwest::header;

use crate::framework::Error;

pub static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

pub fn build_headers_map(
    authorization: Option<&String>,
    extra: Option<&HashMap<String, String>>,
) -> Result<header::HeaderMap, Error> {
    let mut headers = header::HeaderMap::new();

    headers.insert(
        header::CONTENT_TYPE,
        header::HeaderValue::try_from("application/json").map_err(Error::config)?,
    );

    if let Some(auth_value) = &authorization {
        let auth_value = header::HeaderValue::try_from(*auth_value).map_err(Error::config)?;
        headers.insert(header::AUTHORIZATION, auth_value);
    }

    if let Some(custom) = &extra {
        for (name, value) in custom.iter() {
            let name = header::HeaderName::try_from(name).map_err(Error::config)?;
            let value = header::HeaderValue::try_from(value).map_err(Error::config)?;
            headers.insert(name, value);
        }
    }

    Ok(headers)
}
