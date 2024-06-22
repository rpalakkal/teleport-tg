use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::twitter::auth::TwitterTokenPair;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    pub x_id: String,
    pub username: String,
    pub token_pair: TwitterTokenPair,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct InMemoryDB {
    pub oauth_tokens: BTreeMap<String, String>,
    pub access_tokens: BTreeMap<String, User>,
}
