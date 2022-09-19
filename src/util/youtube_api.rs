use anyhow::{anyhow, Context};
use serde_json::json;

const YOUTUBE_API_KEY: &str = "AIzaSyDLqTiych6aAiD05fGSzsqFtCrS6p2GuKY";

pub fn search(query: &str, max_results: usize) -> anyhow::Result<serde_json::Value> {
    let query = json!({
        "part": "snippet",
        "type": "video",
        "maxResults": max_results,
        "key": YOUTUBE_API_KEY,
        "q": query,
    });
    let client = reqwest::blocking::Client::new();
    let response = client
        .get("https://www.googleapis.com/youtube/v3/search")
        .query(&query)
        .send()
        .context("Failed to send HTTP request")?;

    if response.status().is_success() {
        Ok(response.json().unwrap())
    } else {
        Err(anyhow!("API is currently not working"))
    }
}
