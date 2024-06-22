use teloxide::{
    macros::BotCommands,
    requests::{Requester, ResponseResult},
    types::Message,
    utils::command::BotCommands as _,
    Bot,
};

use super::twitter_commands;
use crate::{endpoints::SharedState, twitter};

#[derive(BotCommands, Clone, Debug)]
#[command(
    rename_rule = "lowercase",
    description = "Teleport Telegram Bot commands:"
)]
pub enum BasicCommand {
    #[command(description = "Show this help message")]
    Help,
    #[command(description = "Authenticate a chat with Twitter")]
    Authenticate,
}

pub async fn command_handler(
    bot: Bot,
    shared_state: SharedState,
    msg: Message,
    cmd: BasicCommand,
) -> ResponseResult<()> {
    match cmd {
        BasicCommand::Help => {
            let basic_command_descriptions = BasicCommand::descriptions().to_string();
            let twitter_command_descriptions =
                twitter_commands::TwitterCommand::descriptions().to_string();
            let all_descriptions = format!(
                "{}\n\n{}",
                basic_command_descriptions, twitter_command_descriptions
            );
            bot.send_message(msg.chat.id, all_descriptions).await?;
        }
        BasicCommand::Authenticate => {
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
                let token_pair = twitter::auth::request_oauth_token(chat_id).await.unwrap();
                let url = format!(
                    "https://api.twitter.com/oauth/authenticate?oauth_token={}",
                    token_pair.token.clone()
                );
                let mut db = shared_state.db.lock().await;
                db.oauth_tokens.insert(token_pair.token, token_pair.secret);
                drop(db);
                bot.send_message(msg.chat.id, format!("Please visit: {}", url))
                    .await?;
            }
        }
    };

    Ok(())
}
