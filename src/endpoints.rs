use std::sync::Arc;

use axum::extract::{Query, State};
use serde::Deserialize;
use teloxide::{prelude::Requester, types::ChatId, Bot};
use tokio::sync::Mutex;

use crate::{
    db::{InMemoryDB, User},
    twitter::{auth::authorize_token, builder::TwitterBuilder},
};

#[derive(Deserialize)]
pub struct CallbackQuery {
    oauth_token: String,
    oauth_verifier: String,
    chat_id: String,
}

#[derive(Clone)]
pub struct SharedState {
    pub db: Arc<Mutex<InMemoryDB>>,
    pub bot: Bot,
    pub twitter: TwitterBuilder,
    pub bot_name: String,
}

pub async fn callback(
    State(shared_state): State<SharedState>,
    Query(query): Query<CallbackQuery>,
) -> &'static str {
    let oauth_token = query.oauth_token;
    let oauth_verifier = query.oauth_verifier;
    let chat_id = query.chat_id;

    let mut db = shared_state.db.lock().await;
    let oauth_access_secret = db
        .oauth_tokens
        .remove(&oauth_token)
        .expect("Failed to find oauth_access_secret in database");

    let token_pair = authorize_token(oauth_token, oauth_access_secret, oauth_verifier)
        .await
        .unwrap();
    let x_info = shared_state
        .twitter
        .with_auth(token_pair.clone())
        .get_user_info()
        .await
        .unwrap();
    let user = User {
        x_id: x_info.id.clone(),
        username: x_info.username.clone(),
        token_pair,
    };
    let user_profile_url = format!("https://x.com/{}", x_info.username);
    let msg = format!(
        "Succesfully authenticated user: {}",
        user_profile_url.clone()
    );
    log::info!("{}", msg);
    let tg_chat_id = chat_id.parse::<i64>().unwrap();
    let tg_chat_id = ChatId(tg_chat_id);
    shared_state
        .bot
        .send_message(tg_chat_id, msg)
        .await
        .unwrap();
    db.access_tokens.insert(chat_id, user);
    drop(db);
    "Success"
}
