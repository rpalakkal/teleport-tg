use std::sync::Arc;

use commands::{command_handler, Command};
use db::InMemoryDB;
use endpoints::{callback, SharedState};
use teloxide::{
    dispatching::{HandlerExt, UpdateFilterExt},
    dptree,
    prelude::Dispatcher,
    types::Update,
    Bot,
};
use tokio::sync::Mutex;
mod commands;
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

    let handler = Update::filter_message().branch(
        dptree::entry()
            .filter_command::<Command>()
            .endpoint(command_handler),
    );

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![shared_state])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}
