use axum_sessions::extractors::WritableSession;
use serde::{Deserialize, Serialize};
use strum::Display;

#[derive(Display)]
pub(crate) enum FlashType {
    CreateFormUserValues,
    Alert,
}

pub(crate) fn flash<T>(kind: FlashType, value: T, session: &mut WritableSession)
where
    T: Serialize,
{
    let _ = session.insert(kind.to_string().as_str(), value);

    let mut flash_keys: Vec<String> = session.get("__flash_keys").unwrap_or_default();
    if !flash_keys.contains(&kind.to_string()) {
        flash_keys.push(kind.to_string());
    }

    let _ = session.insert("__flash_keys", flash_keys);

    dbg!(
        kind.to_string().as_str(),
        session.get_raw(kind.to_string().as_str())
    );
}

pub(crate) fn clear_flash(session: &mut WritableSession) {
    let flash_keys: Vec<String> = session.get("__flash_keys").unwrap_or_default();

    for key in flash_keys {
        session.remove(&key);
    }

    session.remove("__flash_keys");
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Alert {
    Success(String),
    Info(String),
    Error(String),
}
