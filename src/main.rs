use std::{
    fs::File,
    io::{LineWriter, Write},
};

use reqwest::{header, ClientBuilder};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let user_name = std::env::args().nth(1).expect("You must provide user name");
    let mut headers = header::HeaderMap::new();
    headers.insert(
        "Accept",
        header::HeaderValue::from_static("application/vnd.github+json"),
    );
    headers.insert(
        "User-Agent",
        header::HeaderValue::from_static("github_followers_check"),
    );
    let client = ClientBuilder::new()
        .default_headers(headers)
        .build()
        .expect("Failed to build client");
    let mut page = 1;
    let mut followers = vec![];
    loop {
        let follower_up_to_100 = client
            .get(format!(
                "https://api.github.com/users/{}/followers",
                user_name
            ))
            .query(&[("page", page), ("per_page", 100)])
            .send()
            .await?
            .json::<Vec<User>>()
            .await?;
        if follower_up_to_100.is_empty() {
            break;
        }
        followers.extend(follower_up_to_100);
        page += 1;
    }
    let mut page = 1;
    let mut following = vec![];
    loop {
        let following_up_to_100 = client
            .get(format!(
                "https://api.github.com/users/{}/following",
                user_name
            ))
            .query(&[("page", page), ("per_page", 100)])
            .send()
            .await?
            .json::<Vec<User>>()
            .await?;
        if following_up_to_100.is_empty() {
            break;
        }
        following.extend(following_up_to_100);
        page += 1;
    }
    let mut following_set = std::collections::HashSet::new();
    for user in following {
        following_set.insert((user.login, user.id));
    }
    let mut followers_set = std::collections::HashSet::new();
    for user in followers {
        followers_set.insert((user.login, user.id));
    }
    let users_who_not_follows_you: Vec<_> = following_set.difference(&followers_set).collect();
    let users_who_you_not_follows: Vec<_> = followers_set.difference(&following_set).collect();
    let f = File::create("README.md").expect("Unable to create README");
    let mut f = LineWriter::new(f);
    f.write_all(
        r"# What's this?
This repository checks the follow-following relationships.
READ.md is generated once per 24 hours and automatically updated.
# How to use
1. Clone this repo."
            .as_bytes(),
    )?;
    if !users_who_you_not_follows.is_empty() {
        f.write_all(b"\n \n --- \n \n # Users who you not follows: \n  \n")?;
        for user in users_who_you_not_follows {
            let user_url = format!("https://github.com/{}/", user.0);
            f.write_all(format!("- [{}]({}) \n", user.0, user_url).as_bytes())?;
        }
    }

    if !users_who_not_follows_you.is_empty() {
        f.write_all(b"# Users who not follows you \n  \n")?;
        for user in users_who_not_follows_you {
            let user_url = format!("https://github.com/{}/", user.0);
            f.write_all(format!("- [{}]({}) \n ", user.0, user_url).as_bytes())?;
        }
    }
    f.flush()?;
    Ok(())
}

#[derive(Debug, serde::Deserialize)]
struct User {
    id: usize,
    login: String,
}
