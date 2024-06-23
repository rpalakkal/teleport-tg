use eyre::OptionExt;
use teloxide::{macros::BotCommands, requests::Requester, types::ChatId, Bot};

use crate::{endpoints::SharedState, twitter::tweet::Tweet};

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

fn twitter_command_message(cmd: TwitterCommand, url: String) -> String {
    match cmd {
        TwitterCommand::Tweet(_) => format!("Tweet sent: {}", url),
        TwitterCommand::Like(_) => "Tweet liked".to_string(),
        TwitterCommand::Retweet(_) => "Tweet retweeted".to_string(),
        TwitterCommand::Reply(_) => format!("Reply sent: {}", url),
        TwitterCommand::Quote(_) => format!("Quote tweet sent: {}", url),
    }
}

pub async fn twitter_command_handler(
    bot: Bot,
    shared_state: SharedState,
    cmd: TwitterCommand,
    chat_id: ChatId,
    media: Option<Vec<u8>>,
) -> eyre::Result<()> {
    let db = shared_state.db.lock().await;
    let user = db
        .access_tokens
        .get(&chat_id.to_string())
        .map(|u| u.clone());
    drop(db);
    if user.is_none() {
        bot.send_message(chat_id, "Please /authenticate first")
            .await?;
        return Ok(());
    }
    let user = user.unwrap();
    let client = shared_state.twitter.with_auth(user.token_pair);
    let media_ids = if let Some(media) = media {
        let media_id = client.upload_media(media).await?;
        vec![media_id]
    } else {
        vec![]
    };
    let id = match cmd.clone() {
        TwitterCommand::Like(tweet_url) | TwitterCommand::Retweet(tweet_url) => {
            let tweet_id = tweet_url
                .rsplit('/')
                .next()
                .ok_or_eyre("Failed to extract tweet_id from tweet_url")?;
            let _ = if let TwitterCommand::Like(_) = cmd {
                client.like(user.x_id, tweet_id.to_string()).await?
            } else {
                client.retweet(user.x_id, tweet_id.to_string()).await?
            };
            "".to_string()
        }
        TwitterCommand::Quote(text) | TwitterCommand::Reply(text) => {
            let (tweet_url, tweet) = text.split_once(' ').ok_or_eyre("Invalid quote command")?;
            let tweet_id = tweet_url
                .rsplit('/')
                .next()
                .ok_or_eyre("Failed to extract tweet_id from tweet_url")?;
            let mut tweet = Tweet::new(tweet.to_string());
            if let TwitterCommand::Quote(_) = cmd {
                tweet.set_quote_tweet_id(tweet_id.to_string());
            } else {
                tweet.set_reply_tweet_id(tweet_id.to_string());
            }
            if !media_ids.is_empty() {
                tweet.set_media_ids(media_ids);
            }
            client.raw_tweet(tweet).await?
        }
        TwitterCommand::Tweet(tweet) => {
            let mut tweet = Tweet::new(tweet);
            if !media_ids.is_empty() {
                tweet.set_media_ids(media_ids);
            }
            client.raw_tweet(tweet).await?
        }
    };

    let url = format!("https://x.com/{}/status/{}", user.username, id);
    bot.send_message(chat_id, twitter_command_message(cmd, url))
        .await?;

    Ok(())
}
