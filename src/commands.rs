use teloxide::{prelude::*, utils::command::BotCommands};

use crate::{endpoints::SharedState, twitter};

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
pub enum Command {
    Help,
    Authenticate,
    Tweet(String),
    Like(String),
}

pub async fn command_handler(
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
        Command::Like(tweet_url) => {
            let db = shared_state.db.lock().await;
            let user = db
                .access_tokens
                .get(&msg.chat.id.to_string())
                .expect("Failed to find access_token in database")
                .clone();
            drop(db);
            let tweet_id = tweet_url
                .rsplit('/')
                .next()
                .expect("Failed to extract tweet_id from tweet_url");
            let _ = twitter::like_tweet(
                user.access_token,
                user.access_secret,
                user.x_id,
                tweet_id.to_string(),
            )
            .await
            .expect("Failed to like tweet");
            bot.send_message(msg.chat.id, "Tweet liked").await?;
        }
    };

    Ok(())
}
