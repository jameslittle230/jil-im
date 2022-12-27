pub(crate) mod fetch;

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default)]
pub(crate) struct State {
    pub(crate) links: HashMap<String, Link>,
    pub(crate) password_hash: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub(crate) struct Link {
    pub(crate) shortname: String,
    pub(crate) longurl: String,
    pub(crate) created_at: String,
    pub(crate) clicks: usize,
}
