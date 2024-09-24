use std::{fs::File, io::BufReader};

use rayon::prelude::*;
use teleport_tg::twitter::{auth::TwitterTokenPair, builder::TwitterBuilder, tweet::Tweet};

#[derive(serde::Deserialize)]
struct Tokens {
    tokens: Vec<TwitterTokenPair>,
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    env_logger::init();

    let app_key = std::env::var("TWITTER_CONSUMER_KEY").expect("TWITTER_CONSUMER_KEY not set");
    let app_secret =
        std::env::var("TWITTER_CONSUMER_SECRET").expect("TWITTER_CONSUMER_SECRET not set");
    // let tweet_count = 5;
    let twitter = TwitterBuilder::new(app_key, app_secret);

    let file = File::open("tokens.json").unwrap();
    let reader = BufReader::new(file);
    let tokens: Tokens = serde_json::from_reader(reader).unwrap();

    let runtime = tokio::runtime::Runtime::new().unwrap();
    tokens.tokens.par_iter().for_each(|token_pair| {
        let client = twitter.with_auth(token_pair.clone());
        // (0..tweet_count).into_par_iter().for_each(|i| {
        //     let tweet = Tweet::new(format!("<>:: ::<> ::<>! {}", i));
        //     let res = runtime.block_on(client.raw_tweet(tweet));
        //     if let Err(e) = res {
        //         log::error!("Error sending tweet: {:?}", e);
        //     }
        // });
        let tweet = Tweet::new(format!("<>:: ::<> ::<>!"));
        let res = runtime.block_on(client.raw_tweet(tweet));
        if let Err(e) = res {
            log::error!("Error sending tweet: {:?}", e);
        }
    });
}
