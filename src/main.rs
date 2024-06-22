use std::sync::Arc;

use db::InMemoryDB;
use endpoints::{callback, SharedState};
use handlers::{
    basic_commands::{command_handler, BasicCommand},
    twitter_commands::{twitter_command_handler, TwitterCommand},
};
use teloxide::{
    dispatching::{HandlerExt, UpdateFilterExt},
    dptree,
    prelude::Dispatcher,
    types::Update,
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

    let app_key = std::env::var("TWITTER_CONSUMER_KEY").expect("TWITTER_CONSUMER_KEY not set");
    let app_secret =
        std::env::var("TWITTER_CONSUMER_SECRET").expect("TWITTER_CONSUMER_SECRET not set");

    let shared_state = SharedState {
        db: Arc::new(Mutex::new(InMemoryDB::default())),
        bot: bot.clone(),
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
        .branch(
            dptree::entry()
                .filter_command::<TwitterCommand>()
                .endpoint(twitter_command_handler),
        );
    // .branch(
    //     dptree::filter(|msg: Message| msg.photo().is_some()).endpoint(
    //         |msg: Message, shared_state: SharedState, bot: Bot| async move {
    //             log::info!(
    //                 "Received a photo from {}",
    //                 msg.from()
    //                     .map(|u| u.full_name())
    //                     .unwrap_or("someone".to_string())
    //             );
    //             if let Some(photos) = msg.photo() {
    //                 // let photo = photos[0].clone();
    //                 let photo = photos.iter().max_by_key(|p| p.file.size).unwrap().clone();
    //                 let file = bot.get_file(&photo.file.id).await?;
    //                 let download = bot.download_file_stream(&file.path);
    //                 let buffer = download
    //                     .fold(Vec::new(), |mut vec, chunk| {
    //                         vec.extend_from_slice(&chunk.unwrap());
    //                         async { vec }
    //                     })
    //                     .await;
    //                 // let base64_string = BASE64_STANDARD.encode(&buffer);
    //                 // log::info!("{}", base64_string);
    //                 let db = shared_state.db.lock().await;
    //                 let user = db
    //                     .access_tokens
    //                     .get(&msg.chat.id.to_string())
    //                     .map(|u| u.clone());
    //                 drop(db);
    //                 if user.is_none() {
    //                     bot.send_message(msg.chat.id, "Please /authenticate first")
    //                         .await?;
    //                     return Ok(());
    //                 }
    //                 let user = user.unwrap();
    //                 let id = twitter::upload_media(
    //                     user.access_token.clone(),
    //                     user.access_secret.clone(),
    //                     buffer,
    //                 )
    //                 .await
    //                 .expect("Failed to upload media");
    //                 twitter::send_tweet(
    //                     user.access_token,
    //                     user.access_secret,
    //                     "Testing media".to_string(),
    //                     Some(vec![id]),
    //                 )
    //                 .await
    //                 .expect("Failed to send tweet");
    //             }
    //             // let bot_name = bot
    //             //     .get_me()
    //             //     .await?
    //             //     .user
    //             //     .username
    //             //     .expect("Bot must have a username");

    //             // let caption: Option<Command> =
    //             //     msg.caption().map(|c| Command::parse(c, &bot_name).unwrap());

    //             // if let Some(caption) = caption {
    //             //     // command_handler(caption, msg, shared_state, bot).await?;
    //             //     log::info!("Received command: {:?}", caption);
    //             // }
    //             Ok(())
    //         },
    //     ),
    // );

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![shared_state])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}
