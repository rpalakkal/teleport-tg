use std::sync::Arc;

use db::InMemoryDB;
use endpoints::{callback, SharedState};
use futures_util::StreamExt;
use handlers::{
    basic_commands::{command_handler, BasicCommand},
    twitter_commands::{twitter_command_handler, TwitterCommand},
};
use teloxide::{
    dispatching::{HandlerExt, UpdateFilterExt},
    dptree,
    net::Download,
    prelude::Dispatcher,
    requests::Requester,
    types::{Message, Update},
    utils::command::BotCommands,
    Bot,
};
use tokio::sync::Mutex;
use twitter::builder::TwitterBuilder;
mod db;
mod endpoints;
mod handlers;
mod twitter;

#[tokio::main]
async fn main() {
    env_logger::init();
    dotenv::dotenv().ok();
    log::info!("Starting command bot...");

    let bot = Bot::from_env();

    let bot_name = bot
        .get_me()
        .await
        .expect("Failed to get bot info")
        .user
        .username
        .expect("Bot must have a username");

    let app_key = std::env::var("TWITTER_CONSUMER_KEY").expect("TWITTER_CONSUMER_KEY not set");
    let app_secret =
        std::env::var("TWITTER_CONSUMER_SECRET").expect("TWITTER_CONSUMER_SECRET not set");

    let shared_state = SharedState {
        db: Arc::new(Mutex::new(InMemoryDB::default())),
        bot: bot.clone(),
        bot_name,
        twitter: TwitterBuilder::new(app_key, app_secret),
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
        .branch(
            dptree::entry()
                .filter_command::<BasicCommand>()
                .endpoint(command_handler),
        )
        .branch(dptree::entry().filter_command::<TwitterCommand>().endpoint(
            |bot: Bot, shared_state: SharedState, msg: Message, cmd: TwitterCommand| async move {
                let res = twitter_command_handler(bot, shared_state, cmd, msg.chat.id, None).await;
                if let Err(e) = res {
                    log::error!("Error handling twitter command: {:?}", e);
                }
                Ok(())
            },
        ))
        .branch(
            dptree::filter(|msg: Message| msg.photo().is_some()).endpoint(
                |bot: Bot, msg: Message, shared_state: SharedState| async move {
                    let photos = msg.photo().unwrap();
                    let photo = photos.iter().max_by_key(|p| p.file.size).unwrap().clone();
                    let file = bot.get_file(&photo.file.id).await?;
                    let download = bot.download_file_stream(&file.path);
                    let buffer = download
                        .fold(Vec::new(), |mut vec, chunk| {
                            vec.extend_from_slice(&chunk.unwrap());
                            async { vec }
                        })
                        .await;

                    let caption: Option<TwitterCommand> = msg
                        .caption()
                        .map(|c| TwitterCommand::parse(c, &shared_state.bot_name).unwrap());

                    if let Some(cmd) = caption {
                        let res = twitter_command_handler(
                            bot,
                            shared_state,
                            cmd,
                            msg.chat.id,
                            Some(buffer),
                        )
                        .await;
                        if let Err(e) = res {
                            log::error!("Error handling twitter command: {:?}", e);
                        }
                    }
                    Ok(())
                },
            ),
        );

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![shared_state])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}
