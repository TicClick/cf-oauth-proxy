use base64::{engine::general_purpose, Engine as _};
use serde::{Deserialize, Serialize};
use worker::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct StateData {
    pub local_port: u16,
    pub nonce: String,
}

pub fn encode_state(state_data: &StateData) -> Result<String> {
    let json = serde_json::to_string(state_data).map_err(|e| Error::RustError(e.to_string()))?;
    Ok(general_purpose::URL_SAFE_NO_PAD.encode(json.as_bytes()))
}

pub fn decode_state(token: &str) -> Result<StateData> {
    let bytes = general_purpose::URL_SAFE_NO_PAD
        .decode(token)
        .map_err(|e| Error::RustError(e.to_string()))?;
    let json = String::from_utf8(bytes).map_err(|e| Error::RustError(e.to_string()))?;
    let state_data = serde_json::from_str(&json).map_err(|e| Error::RustError(e.to_string()))?;
    Ok(state_data)
}
