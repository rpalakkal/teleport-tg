use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct User {
    pub x_id: String,
    pub username: String,
    pub access_token: String,
    pub access_secret: String,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct InMemoryDB {
    pub oauth_tokens: BTreeMap<String, String>,
    pub access_tokens: BTreeMap<String, User>,
}
