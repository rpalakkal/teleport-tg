use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
struct RequestTokenRequestQuery {
    oauth_callback: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
struct RequestTokenResponseBody {
    oauth_token: String,
    oauth_token_secret: String,
    oauth_callback_confirmed: bool,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
struct CallbackUrlQuery {
    oauth_token: String,
    oauth_verifier: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct TwitterTokenPair {
    pub token: String,
    pub secret: String,
}

pub async fn request_oauth_token(chat_id: String) -> eyre::Result<TwitterTokenPair> {
    let app_key = std::env::var("TWITTER_CONSUMER_KEY").expect("TWITTER_CONSUMER_KEY not set");
    let app_secret =
        std::env::var("TWITTER_CONSUMER_SECRET").expect("TWITTER_CONSUMER_SECRET not set");
    let callback_url = std::env::var("CALLBACK_URL").expect("CALLBACK_URL not set");
    let callback_url = format!("{}/callback?chat_id={}", callback_url, chat_id);
    let secrets = reqwest_oauth1::Secrets::new(app_key, app_secret);
    let query = RequestTokenRequestQuery {
        oauth_callback: callback_url.to_string(),
    };
    let response = reqwest_oauth1::Client::new()
        .post("https://api.twitter.com/oauth/request_token")
        .sign(secrets)
        .query(&query)
        .generate_signature()?
        .send()
        .await?;
    let status = response.status();
    if !status.is_success() {
        eyre::bail!(response.text().await?);
    }
    let response_bytes = response.bytes().await?;
    let request_token_body =
        serde_urlencoded::from_bytes::<RequestTokenResponseBody>(&response_bytes)?;
    assert!(request_token_body.oauth_callback_confirmed);
    Ok(TwitterTokenPair {
        token: request_token_body.oauth_token,
        secret: request_token_body.oauth_token_secret,
    })
}

#[derive(Deserialize, Serialize, Debug, Clone)]
struct AccessTokenResponseBody {
    oauth_token: String,
    oauth_token_secret: String,
    user_id: u64,
    screen_name: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
struct AccessTokenRequestQuery {
    oauth_verifier: String,
}

pub async fn authorize_token(
    oauth_token: String,
    oauth_token_secret: String,
    oauth_verifier: String,
) -> eyre::Result<TwitterTokenPair> {
    let app_key = std::env::var("TWITTER_CONSUMER_KEY").expect("TWITTER_CONSUMER_KEY not set");
    let app_secret =
        std::env::var("TWITTER_CONSUMER_SECRET").expect("TWITTER_CONSUMER_SECRET not set");

    let query = AccessTokenRequestQuery { oauth_verifier };

    let secrets =
        reqwest_oauth1::Secrets::new(app_key, app_secret).token(oauth_token, oauth_token_secret);

    let response = reqwest_oauth1::Client::new()
        .post("https://api.twitter.com/oauth/access_token")
        .sign(secrets)
        .query(&query)
        .generate_signature()?
        .send()
        .await?;

    let status = response.status();
    if !status.is_success() {
        eyre::bail!(response.text().await?);
    }
    let response_bytes = response.bytes().await?;

    let access_token_body =
        serde_urlencoded::from_bytes::<AccessTokenResponseBody>(&response_bytes)?;

    Ok(TwitterTokenPair {
        token: access_token_body.oauth_token,
        secret: access_token_body.oauth_token_secret,
    })
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[tokio::test]
//     #[ignore]
//     async fn e2e_oauth_test() {
//         env_logger::init();
//         dotenv::dotenv().ok();
//         let tokens = request_oauth_token(1.to_string()).await.unwrap();
//         // log::info!("{:?}", tokens);
//         let url = format!(
//             "https://api.twitter.com/oauth/authenticate?oauth_token={}",
//             tokens.token.clone()
//         );
//         log::info!("Please visit: {}", url);
//         let mut callback_url = String::new();
//         std::io::stdin().read_line(&mut callback_url).unwrap();
//         let url = url::Url::parse(&callback_url).unwrap();
//         let callback_url_query = url.query().unwrap_or_default();
//         let callback_url_query: CallbackUrlQuery = serde_qs::from_str(callback_url_query).unwrap();
//         let tokens = authorize_token(
//             tokens.token,
//             tokens.secret,
//             callback_url_query.oauth_verifier,
//         )
//         .await
//         .unwrap();
//         // log::info!("{:?}", tokens);
//     }
// }
