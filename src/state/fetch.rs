use std::sync::Arc;
use tokio::sync::Mutex;

use crate::api_client::ApiClient;

use super::State;

pub(crate) async fn fetch_state(state: &Arc<Mutex<State>>) {
    let mut state = state.lock().await;
    let result = ApiClient::new().get_entries().await;

    match result {
        Ok(links) => {
            for link in links.items {
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
