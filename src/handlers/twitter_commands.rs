use teloxide::{
    macros::BotCommands,
    requests::{Requester, ResponseResult},
    types::Message,
    Bot,
};

use crate::endpoints::SharedState;

#[derive(BotCommands, Clone, Debug)]
#[command(rename_rule = "lowercase")]
pub enum TwitterCommand {
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

pub async fn twitter_command_handler(
    bot: Bot,
    shared_state: SharedState,
    msg: Message,
    cmd: TwitterCommand,
) -> ResponseResult<()> {
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
    let client = shared_state.twitter.with_auth(user.token_pair);
    match cmd.clone() {
        TwitterCommand::Like(tweet_url) | TwitterCommand::Retweet(tweet_url) => {
            let tweet_id = tweet_url
                .rsplit('/')
                .next()
                .expect("Failed to extract tweet_id from tweet_url");
            match cmd {
                TwitterCommand::Like(_) => {
                    let _ = client
                        .like_tweet(user.x_id, tweet_id.to_string())
                        .await
                        .expect("Failed to like tweet");
                    bot.send_message(msg.chat.id, "Tweet liked").await?;
                }
                TwitterCommand::Retweet(_) => {
                    let _ = client
                        .retweet(user.x_id, tweet_id.to_string())
                        .await
                        .expect("Failed to retweet tweet");
                    bot.send_message(msg.chat.id, "Tweet retweeted").await?;
                }
                _ => {}
            }
        }
        TwitterCommand::Quote(text) | TwitterCommand::Reply(text) => {
            let (tweet_url, tweet) = text.split_once(' ').expect("Invalid quote command");
            let tweet_id = tweet_url
                .rsplit('/')
                .next()
                .expect("Failed to extract tweet_id from tweet_url");
            match cmd {
                TwitterCommand::Quote(_) => {
                    let id = client
                        .quote(tweet.to_string(), tweet_id.to_string())
                        .await
                        .expect("Failed to quote tweet");
                    let url = format!("https://x.com/{}/status/{}", user.username, id);
                    bot.send_message(msg.chat.id, format!("Quote tweet sent: {}", url))
                        .await?;
                }
                TwitterCommand::Reply(_) => {
                    let id = client
                        .reply(tweet.to_string(), tweet_id.to_string())
                        .await
                        .expect("Failed to reply tweet");
                    let url = format!("https://x.com/{}/status/{}", user.username, id);
                    bot.send_message(msg.chat.id, format!("Reply sent: {}", url))
                        .await?;
                }
                _ => {}
            }
        }
        TwitterCommand::Tweet(tweet) => {
            let id = client.tweet(tweet).await.expect("Failed to send tweet");
            let url = format!("https://x.com/{}/status/{}", user.username, id);
            bot.send_message(msg.chat.id, format!("Quote tweet sent: {}", url))
                .await?;
        }
    };

    Ok(())
}
