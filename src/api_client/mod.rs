use anyhow::{Context, Error, Result};
use reqwest::Response;
use serde::{Deserialize, Serialize};

use crate::state::Link;

#[derive(Serialize, Deserialize)]
struct ApiError {
    error: bool,
    code: String,
    message: String,
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum CreateEntryApiResponse {
    Success(Link),
    Error(ApiError),
}

pub(crate) struct ApiClient {
    jil_api_url: String,
    jil_api_admin_bearer_token: String,
}

impl ApiClient {
    pub(crate) fn new() -> Self {
        Self {
            jil_api_url: std::env::var("JIL_API_URL").unwrap(),
            jil_api_admin_bearer_token: std::env::var("JIL_API_ADMIN_BEARER_TOKEN").unwrap(),
        }
    }

    async fn post(&self, path: &str, body: &serde_json::Value) -> Result<Response> {
        let client = reqwest::Client::new();
        let res = client
            .post(format!("{}/{}", self.jil_api_url, path))
            .json(body)
            .header("User-Agent", "jil-im/0.1.0")
            .header(
                "Authorization",
                format!("Bearer {}", self.jil_api_admin_bearer_token),
            )
            .send()
            .await?;

        Ok(res)
    }

    async fn get(&self, path: &str) -> Result<Response> {
        let client = reqwest::Client::new();
        let res = client
            .get(format!("{}/{}", self.jil_api_url, path))
            .header("User-Agent", "jil-im/0.1.0")
            .header(
                "Authorization",
                format!("Bearer {}", self.jil_api_admin_bearer_token),
            )
            .send()
            .await?;

        Ok(res)
    }

    pub(crate) async fn create_entry(&self, shortname: &str, longurl: &str) -> Result<Link> {
        let resp = self
            .post(
                "shortener/entries",
                &serde_json::json!({
                    "shortname": shortname,
                    "longurl": longurl,
                }),
            )
            .await?;

        let res = resp.json::<CreateEntryApiResponse>().await?;

        match res {
            CreateEntryApiResponse::Success(link) => Ok(link),
            CreateEntryApiResponse::Error(err) => Err(Error::msg(err.message)),
        }
    }

    pub(crate) async fn delete_entry(&self, shortname: &str) -> Result<()> {
        self.post(
            &format!("shortener/entries/{}/delete", shortname),
            &serde_json::json!({}),
        )
        .await?;

        Ok(())
    }
}
