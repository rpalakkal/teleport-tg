use std::sync::Arc;

use db::InMemoryDB;
use endpoints::{callback, SharedState};
use teloxide::{prelude::*, utils::command::BotCommands};
use tokio::sync::Mutex;
mod db;
mod endpoints;
mod twitter;

#[tokio::main]
async fn main() {
    env_logger::init();
    dotenv::dotenv().ok();
    log::info!("Starting command bot...");

    let bot = Bot::from_env();

    let shared_state = SharedState {
        db: Arc::new(Mutex::new(InMemoryDB::default())),
        bot: bot.clone(),
    };

    let app = axum::Router::new()
        .route("/callback", axum::routing::get(callback))
        // .layer(CorsLayer::very_permissive())
        .with_state(shared_state.clone());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let handler = Update::filter_message()
        .branch(dptree::entry().filter_command::<Command>().endpoint(answer));

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![shared_state])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
enum Command {
    Help,
    Authenticate,
    Tweet(String),
}

async fn answer(
    bot: Bot,
    shared_state: SharedState,
    msg: Message,
    cmd: Command,
) -> ResponseResult<()> {
    match cmd {
        Command::Help => {
            bot.send_message(msg.chat.id, Command::descriptions().to_string())
                .await?;
        }
        Command::Tweet(tweet) => {
            let db = shared_state.db.lock().await;
            let user = db
                .access_tokens
                .get(&msg.chat.id.to_string())
                .expect("Failed to find access_token in database")
                .clone();
            drop(db);
            let id = twitter::send_tweet(user.access_token, user.access_secret, tweet)
                .await
                .expect("Failed to send tweet");
            let url = format!("https://x.com/{}/status/{}", user.username, id);
            bot.send_message(msg.chat.id, format!("Tweet sent: {}", url))
                .await?;
        }
        Command::Authenticate => {
            let chat_id = msg.chat.id.to_string();
            let db = shared_state.db.lock().await;
            if db.access_tokens.contains_key(&chat_id) {
                let user = db.access_tokens.get(&chat_id).unwrap();
                let user_profile_url = format!("https://x.com/{}", user.username);
                let to_send = format!(
                    "You are already authenticated as: {}",
                    user_profile_url.clone()
                );
                log::info!("{}", to_send);
                bot.send_message(msg.chat.id, to_send).await?;
                drop(db);
            } else {
                drop(db);
                let (oauth_token, oauth_token_secret) =
                    twitter::request_oauth_token(chat_id).await.unwrap();
                let url = format!(
                    "https://api.twitter.com/oauth/authenticate?oauth_token={}",
                    oauth_token.clone()
                );
                let mut db = shared_state.db.lock().await;
                db.oauth_tokens.insert(oauth_token, oauth_token_secret);
                drop(db);
                bot.send_message(msg.chat.id, format!("Please visit: {}", url))
                    .await?;
            }
        }
    };

    Ok(())
}
