use teloxide::{prelude::*, utils::command::BotCommands};

use crate::{endpoints::SharedState, twitter};

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "Teleport Telegram Bot commands:"
)]
pub enum Command {
    #[command(description = "Show this help message")]
    Help,
    #[command(description = "Authenticate a chat with Twitter")]
    Authenticate,
    #[command(description = "Post a tweet by providing the tweet text")]
    Tweet(String),
    #[command(description = "Like a tweet by providing the tweet URL")]
    Like(String),
    #[command(description = "Retweet a tweet by providing the tweet URL")]
    Retweet(String),
    #[command(description = "Reply to a tweet by providing the tweet URL and the reply text")]
    Reply(String),
    #[command(description = "Quote a tweet by providing the tweet URL and the tweet text")]
    Quote(String),
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
                .map(|u| u.clone());
            drop(db);
            if user.is_none() {
                bot.send_message(msg.chat.id, "Please /authenticate first")
                    .await?;
                return Ok(());
            }
            let user = user.unwrap();
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
                .map(|u| u.clone());
            drop(db);
            if user.is_none() {
                bot.send_message(msg.chat.id, "Please /authenticate first")
                    .await?;
                return Ok(());
            }
            let user = user.unwrap();
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
        Command::Retweet(tweet_url) => {
            let db = shared_state.db.lock().await;
            let user = db
                .access_tokens
                .get(&msg.chat.id.to_string())
                .map(|u| u.clone());
            drop(db);
            if user.is_none() {
                bot.send_message(msg.chat.id, "Please /authenticate first")
                    .await?;
                return Ok(());
            }
            let user = user.unwrap();
            let tweet_id = tweet_url
                .rsplit('/')
                .next()
                .expect("Failed to extract tweet_id from tweet_url");
            let _ = twitter::retweet(
                user.access_token,
                user.access_secret,
                user.x_id,
                tweet_id.to_string(),
            )
            .await
            .expect("Failed to retweet tweet");
            bot.send_message(msg.chat.id, "Tweet retweeted").await?;
        }
        Command::Reply(text) => {
            let (tweet_url, reply_text) = text.split_once(' ').expect("Invalid reply command");
            let db = shared_state.db.lock().await;
            let user = db
                .access_tokens
                .get(&msg.chat.id.to_string())
                .map(|u| u.clone());
            drop(db);
            if user.is_none() {
                bot.send_message(msg.chat.id, "Please /authenticate first")
                    .await?;
                return Ok(());
            }
            let user = user.unwrap();
            let tweet_id = tweet_url
                .rsplit('/')
                .next()
                .expect("Failed to extract tweet_id from tweet_url");
            let id = twitter::reply(
                user.access_token,
                user.access_secret,
                reply_text.to_string(),
                tweet_id.to_string(),
            )
            .await
            .expect("Failed to reply to tweet");
            let url = format!("https://x.com/{}/status/{}", user.username, id);
            bot.send_message(msg.chat.id, format!("Reply sent: {}", url))
                .await?;
        }
        Command::Quote(text) => {
            let (tweet_url, tweet) = text.split_once(' ').expect("Invalid quote command");
            let db = shared_state.db.lock().await;
            let user = db
                .access_tokens
                .get(&msg.chat.id.to_string())
                .map(|u| u.clone());
            drop(db);
            if user.is_none() {
                bot.send_message(msg.chat.id, "Please /authenticate first")
                    .await?;
                return Ok(());
            }
            let user = user.unwrap();
            let tweet_id = tweet_url
                .rsplit('/')
                .next()
                .expect("Failed to extract tweet_id from tweet_url");
            let id = twitter::quote_tweet(
                user.access_token,
                user.access_secret,
                tweet.to_string(),
                tweet_id.to_string(),
            )
            .await
            .expect("Failed to quote tweet");
            let url = format!("https://x.com/{}/status/{}", user.username, id);
            bot.send_message(msg.chat.id, format!("Quote tweet sent: {}", url))
                .await?;
        }
    };

    Ok(())
}
