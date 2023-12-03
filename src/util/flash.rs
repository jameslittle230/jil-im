use serde::{Deserialize, Serialize};
use strum::Display;
use tower_sessions::Session;

#[derive(Display)]
pub(crate) enum FlashType {
    CreateFormUserValues,
    Alert,
}

pub(crate) fn flash<T>(kind: FlashType, value: T, session: &mut Session)
where
    T: Serialize,
{
    let _ = session.insert(kind.to_string().as_str(), value);

    let mut flash_keys: Vec<String> = session
        .get("__flash_keys")
        .expect("infallible")
        .unwrap_or_default();
    if !flash_keys.contains(&kind.to_string()) {
        flash_keys.push(kind.to_string());
    }

    let _ = session.insert("__flash_keys", flash_keys);
}

pub(crate) fn flash_error_alert(message: String, session: &mut Session) {
    flash(FlashType::Alert, Alert::Error(message), session);
}

pub(crate) fn flash_success_alert(message: String, session: &mut Session) {
    flash(FlashType::Alert, Alert::Success(message), session);
}

pub(crate) fn flash_info_alert(message: String, session: &mut Session) {
    flash(FlashType::Alert, Alert::Info(message), session);
}

pub(crate) fn clear_flash(session: &mut Session) {
    let flash_keys: Vec<String> = session
        .get("__flash_keys")
        .expect("infallible")
        .unwrap_or_default();

    for key in flash_keys {
        let _ = session.remove::<String>(&key);
    }

    let _ = session.remove::<String>("__flash_keys");
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Alert {
    Success(String),
    Info(String),
    Error(String),
}
