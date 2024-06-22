use oauth1_request::signature_method::HmacSha1;
use reqwest_oauth1::{Client, OAuthClientProvider, Secrets, Signer};

use super::auth::TwitterTokenPair;

#[derive(Debug, Clone)]
pub struct TwitterBuilder {
    pub consumer_key: String,
    pub consumer_secret: String,
}

pub struct TwitterClient<'a> {
    pub client: Client<Signer<'a, Secrets<'a>, HmacSha1>>,
}

impl TwitterBuilder {
    pub fn new(consumer_key: String, consumer_secret: String) -> Self {
        Self {
            consumer_key,
            consumer_secret,
        }
    }

    pub fn with_auth(&self, tokens: TwitterTokenPair) -> TwitterClient {
        let secrets = Secrets::new(self.consumer_key.clone(), self.consumer_secret.clone())
            .token(tokens.token, tokens.secret);

        let client = reqwest::Client::new();
        // client.oauth1(secrets)
        TwitterClient {
            client: client.oauth1(secrets),
        }
    }
}
