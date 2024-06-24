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

impl InMemoryDB {
    pub fn save(&self, path: &str) -> eyre::Result<()> {
        let file = std::fs::File::create(path)?;
        bincode::serialize_into(file, self)?;
        Ok(())
    }

    pub fn load(path: &str) -> eyre::Result<Self> {
        let file = std::fs::File::open(path)?;
        let db = bincode::deserialize_from(file)?;
        Ok(db)
    }

    pub fn load_or_create(path: &str) -> Self {
        match Self::load(path) {
            Ok(db) => {
                log::info!("Loaded database from {}", path);
                db
            }
            Err(_) => {
                log::info!("Creating new database");
                Self::default()
            }
        }
    }
}
