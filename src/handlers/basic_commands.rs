use teloxide::{
    macros::BotCommands,
    requests::{Requester, ResponseResult},
    types::Message,
    utils::command::BotCommands as _,
    Bot,
};

use super::twitter_commands;
use crate::{
    endpoints::{complete_auth_flow, CallbackQuery, SharedState},
    twitter,
};

#[derive(BotCommands, Clone, Debug)]
#[command(
    rename_rule = "lowercase",
    description = "Teleport Telegram Bot commands:"
)]
pub enum BasicCommand {
    #[command(description = "Show this help message")]
    Help,
    #[command(description = "Get the current logged in user")]
    Account,
    #[command(description = "Remove the authenticated Twitter account from the chat")]
    Logout,
    #[command(description = "Authenticate a chat with Twitter")]
    Auth,
    #[command(description = "Pass the 0.0.0.0:3000 callback URL for auth completion")]
    Prank(String),
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
        BasicCommand::Auth => {
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
        BasicCommand::Prank(url) => {
            let url = url::Url::parse(&url).expect("Failed to parse URL");
            let query_params = url.query().expect("Failed to get query params");
            let callback_query: CallbackQuery =
                serde_qs::from_str(query_params).expect("Failed to parse query");
            complete_auth_flow(shared_state, callback_query)
                .await
                .expect("Failed to complete auth flow");
        }
        BasicCommand::Logout => {
            let chat_id = msg.chat.id.to_string();
            let mut db = shared_state.db.lock().await;
            db.access_tokens.remove(&chat_id);
            drop(db);
            bot.send_message(msg.chat.id, "Successfully logged out")
                .await?;
        }
        BasicCommand::Account => {
            let chat_id = msg.chat.id.to_string();
            let db = shared_state.db.lock().await;
            let user = db.access_tokens.get(&chat_id).map(|u| u.clone());
            drop(db);
            if user.is_none() {
                bot.send_message(msg.chat.id, "No Twitter account is currently logged in.")
                    .await?;
                return Ok(());
            }
            let user = user.unwrap();
            let user_profile_url = format!("https://x.com/{}", user.username);
            let to_send = format!("You are authenticated as: {}", user_profile_url);
            bot.send_message(msg.chat.id, to_send).await?;
        }
    };

    Ok(())
}
