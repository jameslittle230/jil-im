use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};

use super::{Link, State};

#[derive(Serialize, Deserialize, Debug)]
struct EntriesResponse {
    items: Vec<Link>,
}

pub(crate) async fn fetch_state(state: &Arc<Mutex<State>>) {
    let client = reqwest::Client::new();
    let result = client
        .get(format!(
            "{}/shortener/entries",
            std::env::var("JIL_API_URL").unwrap()
        ))
        .header("User-Agent", "jil-im/0.1.0")
        .send()
        .await;

    let mut state = state.lock().unwrap();

    match result {
        Ok(response) => {
            let value: EntriesResponse = response.json().await.unwrap();

            for link in &value.items {
                state.links.insert(link.shortname.clone(), link.clone());
            }
        }
        Err(err) => {
            panic!("{}", err.to_string());
        }
    };

    state.password_hash =
        bcrypt::hash(std::env::var("WEB_PASSWORD").unwrap(), bcrypt::DEFAULT_COST).unwrap();
}
