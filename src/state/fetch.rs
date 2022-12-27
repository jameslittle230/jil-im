use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};

use super::{Link, State};

#[derive(Serialize, Deserialize)]
struct EntriesResponse {
    items: Vec<Link>,
}

pub(crate) async fn fetch_state(state: &Arc<Mutex<State>>) {
    let client = reqwest::Client::new();
    let result = client
        .get(format!(
            "{}/shortener/entries",
            dbg!(std::env::var("JIL_API_URL")).unwrap()
        ))
        .send()
        .await;

    dbg!(&result);

    let mut state = state.lock().unwrap();

    match result {
        Ok(response) => {
            let value: EntriesResponse = response.json().await.unwrap();

            for link in &value.items {
                state.links.insert(link.shortname.clone(), link.clone());
            }
        }
        Err(err) => {
            dbg!(err);
        }
    };

    state.password_hash =
        bcrypt::hash(std::env::var("WEB_PASSWORD").unwrap(), bcrypt::DEFAULT_COST).unwrap();
}
