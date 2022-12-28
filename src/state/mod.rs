pub(crate) mod fetch;

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize)]
pub(crate) struct State {
    pub(crate) links: HashMap<String, Link>,
    pub(crate) password_hash: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub(crate) struct Link {
    pub(crate) shortname: String,
    pub(crate) longurl: String,
    pub(crate) created_at: DateTime<Utc>,

    #[serde(default)]
    pub(crate) clicks: usize,
}

impl Link {
    pub(crate) fn created_at_timeago(&self) -> String {
        let formatter = timeago::Formatter::new();
        formatter.convert_chrono(self.created_at, Utc::now())
    }

    pub(crate) fn created_at_iso8601(&self) -> String {
        self.created_at.format("%+").to_string()
    }
}
