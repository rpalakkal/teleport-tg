use std::{fs::File, io::BufReader};

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
    let tweet_count = 5;
    let twitter = TwitterBuilder::new(app_key, app_secret);

    let file = File::open("tokens.json").unwrap();
    let reader = BufReader::new(file);
    let tokens: Tokens = serde_json::from_reader(reader).unwrap();

    for token_pair in tokens.tokens {
        let client = twitter.with_auth(token_pair);
        // let tweet = Tweet::new("Wow aaa!".to_string());
        for i in 0..tweet_count {
            let tweet = Tweet::new(format!("Wow aaa! {}", i));
            let _ = client.raw_tweet(tweet).await;
        }
    }
}
