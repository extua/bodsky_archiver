use bodsky_archiver::convert_at_uri_to_url;
use core::panic;
use std::borrow::Borrow;
use serde::{Deserialize, Serialize};
use serde_json::Value;
mod config;

fn get_posts_number() -> usize {
    #[derive(Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Profile {
        posts_count: usize,
    }

    #[tokio::main]
    async fn request_profile_from_api() -> Result<Profile, reqwest::Error> {
        let raw_response: Result<Profile, reqwest::Error> = reqwest::Client::new()
            .get("https://public.api.bsky.app/xrpc/app.bsky.actor.getProfile")
            .query(&[("actor", config::ACCOUNT_DID)])
            .send()
            .await?
            .json::<Profile>()
            .await;
        raw_response
    }
    let response: Profile = match request_profile_from_api() {
        Ok(response) => response,
        Err(error) => panic!("Failed to get or parse API response: {error:?}"),
    };
    response.posts_count
}

fn collect_api_responses(total_posts: usize) -> Vec<String> {
    // This loop tracks the number of posts remaining and the number
    // to make in each api call
    let api_calls_needed: usize = total_posts.div_euclid(config::POSTS_PER_REQUEST) + 1;
    let mut current_call: usize = 1;
    let mut posts_remaining: usize = total_posts;
    let mut posts_to_request: usize = config::POSTS_PER_REQUEST;
    let mut cursor: String = "".to_string();
    let mut feed: Vec<String> = Vec::with_capacity(total_posts);
    while current_call <= api_calls_needed {
        if posts_remaining < config::POSTS_PER_REQUEST {
            posts_to_request = posts_remaining
        }
        println!("requesting {} posts", posts_to_request);
        let bulk_posts: AuthorFeed = request_bulk_posts_from_api(posts_to_request, &cursor);
        // update the cursor value
        cursor = bulk_posts.cursor;
        for post in bulk_posts.feed {
            let at_uri: &str = post["post"]["uri"].as_str().unwrap();
            let http_url: String = convert_at_uri_to_url(at_uri);
            feed.push(http_url);
        }
        current_call += 1;
        posts_remaining -= posts_to_request;
    }

    #[derive(Serialize, Deserialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct AuthorFeed {
        cursor: String,
        feed: Vec<Value>,
    }

    #[tokio::main]
    async fn request_bulk_posts_from_api(posts_to_request: usize, cursor: &str) -> AuthorFeed {
        let posts_per_request_str: String = posts_to_request.to_string();

        let raw_response: Result<AuthorFeed, reqwest::Error> = reqwest::Client::new()
            .get("https://public.api.bsky.app/xrpc/app.bsky.feed.getAuthorFeed")
            .query(&[
                ("actor", config::ACCOUNT_DID),
                ("limit", &posts_per_request_str),
                ("cursor", &cursor),
            ])
            .send()
            .await
            .unwrap()
            .json::<AuthorFeed>()
            .await;
        let response: AuthorFeed = match raw_response {
            Ok(response) => response,
            Err(error) => panic!("Failed to get or parse API response: {error:?}"),
        };
        response
    }

feed
}

fn main() {
    let total_posts: usize = get_posts_number();
    println!("there are {} posts to request", total_posts);
    let feed_urls = collect_api_responses(total_posts);
    println!("collected {} posts", feed_urls.len());
}
