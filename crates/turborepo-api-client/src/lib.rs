#![feature(async_closure)]

use std::{env, future::Future};

use reqwest::{Method, RequestBuilder, Response, StatusCode, Url};
use serde::{Deserialize, Serialize};

pub use crate::error::{Error, Result};
use crate::retry::retry_future;

mod error;
mod retry;

#[derive(Debug, Clone, Deserialize)]
pub struct VerifiedSsoUser {
    pub token: String,
    pub team_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VerificationResponse {
    pub token: String,
    pub team_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CachingStatus {
    Disabled,
    Enabled,
    OverLimit,
    Paused,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachingStatusResponse {
    pub status: CachingStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactResponse {
    pub duration: u64,
    pub expected_tag: Option<String>,
    pub body: Vec<u8>,
}

/// Membership is the relationship between the logged-in user and a particular
/// team
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Membership {
    role: Role,
}

impl Membership {
    #[allow(dead_code)]
    pub fn new(role: Role) -> Self {
        Self { role }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Role {
    Member,
    Owner,
    Viewer,
    Developer,
    Billing,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Team {
    pub id: String,
    pub slug: String,
    pub name: String,
    #[serde(rename = "createdAt")]
    pub created_at: u64,
    pub created: chrono::DateTime<chrono::Utc>,
    pub membership: Membership,
}

impl Team {
    pub fn is_owner(&self) -> bool {
        matches!(self.membership.role, Role::Owner)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Space {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamsResponse {
    pub teams: Vec<Team>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpacesResponse {
    pub spaces: Vec<Space>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: String,
    pub name: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserResponse {
    pub user: User,
}

pub struct PreflightResponse {
    location: Url,
    allow_auth: bool,
}

pub struct APIClient {
    client: reqwest::Client,
    base_url: String,
    user_agent: String,
}

impl APIClient {
    pub async fn get_user(&self, token: &str) -> Result<UserResponse> {
        let response = self
            .make_retryable_request(async || {
                let url = self.make_url("/v2/user");
                let request_builder = self
                    .client
                    .get(url)
                    .header("User-Agent", self.user_agent.clone())
                    .header("Authorization", format!("Bearer {}", token))
                    .header("Content-Type", "application/json");

                Ok(request_builder.send().await?)
            })
            .await?
            .error_for_status()?;

        Ok(response.json().await?)
    }

    pub async fn get_teams(&self, token: &str) -> Result<TeamsResponse> {
        let response = self
            .make_retryable_request(async || {
                let request_builder = self
                    .client
                    .get(self.make_url("/v2/teams?limit=100"))
                    .header("User-Agent", self.user_agent.clone())
                    .header("Content-Type", "application/json")
                    .header("Authorization", format!("Bearer {}", token));

                Ok(request_builder.send().await?)
            })
            .await?
            .error_for_status()?;

        Ok(response.json().await?)
    }

    pub async fn get_team(&self, token: &str, team_id: &str) -> Result<Option<Team>> {
        let response = self
            .client
            .get(self.make_url("/v2/team"))
            .query(&[("teamId", team_id)])
            .header("User-Agent", self.user_agent.clone())
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await?
            .error_for_status()?;

        Ok(response.json().await?)
    }

    fn add_team_params(
        mut request_builder: RequestBuilder,
        team_id: &str,
        team_slug: Option<&str>,
    ) -> RequestBuilder {
        if let Some(slug) = team_slug {
            request_builder = request_builder.query(&[("teamSlug", slug)]);
        }
        if team_id.starts_with("team_") {
            request_builder = request_builder.query(&[("teamId", team_id)]);
        }

        request_builder
    }

    pub async fn get_caching_status(
        &self,
        token: &str,
        team_id: &str,
        team_slug: Option<&str>,
    ) -> Result<CachingStatusResponse> {
        let response = self
            .make_retryable_request(async || {
                let request_builder = self
                    .client
                    .get(self.make_url("/v8/artifacts/status"))
                    .header("User-Agent", self.user_agent.clone())
                    .header("Content-Type", "application/json")
                    .header("Authorization", format!("Bearer {}", token));

                let request_builder = Self::add_team_params(request_builder, team_id, team_slug);
                Ok(request_builder.send().await?)
            })
            .await?
            .error_for_status()?;

        Ok(response.json().await?)
    }

    pub async fn get_spaces(&self, token: &str, team_id: Option<&str>) -> Result<SpacesResponse> {
        // create url with teamId if provided
        let endpoint = match team_id {
            Some(team_id) => format!("/v0/spaces?limit=100&teamId={}", team_id),
            None => "/v0/spaces?limit=100".to_string(),
        };

        let response = self
            .make_retryable_request(|| {
                let request_builder = self
                    .client
                    .get(self.make_url(endpoint.as_str()))
                    .header("User-Agent", self.user_agent.clone())
                    .header("Content-Type", "application/json")
                    .header("Authorization", format!("Bearer {}", token));

                request_builder.send()
            })
            .await?
            .error_for_status()?;

        response.json().await.map_err(|err| {
            anyhow!(
                "Error getting spaces: {}",
                err.status()
                    .and_then(|status| status.canonical_reason())
                    .unwrap_or(&err.to_string())
            )
        })
    }

    pub async fn verify_sso_token(&self, token: &str, token_name: &str) -> Result<VerifiedSsoUser> {
        let response = self
            .make_retryable_request(async || {
                let request_builder = self
                    .client
                    .get(self.make_url("/registration/verify"))
                    .query(&[("token", token), ("tokenName", token_name)])
                    .header("User-Agent", self.user_agent.clone());

                Ok(request_builder.send().await?)
            })
            .await?
            .error_for_status()?;

        let verification_response: VerificationResponse = response.json().await?;

        Ok(VerifiedSsoUser {
            token: verification_response.token,
            team_id: verification_response.team_id,
        })
    }

    const RETRY_MAX: u32 = 2;

    async fn make_retryable_request<F: Future<Output = Result<reqwest::Response>>>(
        &self,
        request_builder: impl Fn() -> F,
    ) -> Result<reqwest::Response> {
        retry_future(Self::RETRY_MAX, request_builder, Self::should_retry_request).await
    }

    fn should_retry_request(error: &Error) -> bool {
        if let Error::ReqwestError(reqwest_error) = error {
            if let Some(status) = reqwest_error.status() {
                if status == StatusCode::TOO_MANY_REQUESTS {
                    return true;
                }

                if status.as_u16() >= 500 && status.as_u16() != 501 {
                    return true;
                }
            }
        }

        false
    }

    pub async fn fetch_artifact(
        &self,
        hash: &str,
        token: &str,
        team_id: &str,
        team_slug: Option<&str>,
        use_preflight: bool,
    ) -> Result<Response> {
        let mut request_url = self.make_url(&format!("/v8/artifacts/{}", hash));
        let mut allow_auth = true;

        if use_preflight {
            let preflight_response = self
                .do_preflight(token, &request_url, "GET", "Authorization, User-Agent")
                .await?;

            allow_auth = preflight_response.allow_auth;
            request_url = preflight_response.location.to_string();
        };

        let response = self
            .make_retryable_request(async || {
                let mut request_builder = self
                    .client
                    .get(&request_url)
                    .header("User-Agent", self.user_agent.clone());

                if allow_auth {
                    request_builder =
                        request_builder.header("Authorization", format!("Bearer {}", token));
                }

                request_builder = Self::add_team_params(request_builder, team_id, team_slug);

                Ok(request_builder.send().await?)
            })
            .await?
            .error_for_status()?;

        Ok(response)
    }

    pub async fn do_preflight(
        &self,
        token: &str,
        request_url: &str,
        request_method: &str,
        request_headers: &str,
    ) -> Result<PreflightResponse> {
        let response = self
            .make_retryable_request(async || {
                let request_builder = self
                    .client
                    .request(Method::OPTIONS, request_url)
                    .header("User-Agent", self.user_agent.clone())
                    .header("Access-Control-Request-Method", request_method)
                    .header("Access-Control-Request-Headers", request_headers)
                    .header("Authorization", format!("Bearer {}", token));

                Ok(request_builder.send().await?)
            })
            .await?;

        let headers = response.headers();
        let location = if let Some(location) = headers.get("Location") {
            let location = location.to_str()?;
            Url::parse(location)?
        } else {
            response.url().clone()
        };

        let allowed_headers = headers
            .get("Access-Control-Allow-Headers")
            .map_or("", |h| h.to_str().unwrap_or(""));

        let allow_auth = allowed_headers.to_lowercase().contains("authorization");

        Ok(PreflightResponse {
            location,
            allow_auth,
        })
    }

    pub fn new(base_url: impl AsRef<str>, timeout: u64, version: &'static str) -> Result<Self> {
        let client = if timeout != 0 {
            reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(timeout))
                .build()?
        } else {
            reqwest::Client::builder().build()?
        };

        let user_agent = format!(
            "turbo {} {} {} {}",
            version,
            rustc_version_runtime::version(),
            env::consts::OS,
            env::consts::ARCH
        );
        Ok(APIClient {
            client,
            base_url: base_url.as_ref().to_string(),
            user_agent,
        })
    }

    fn make_url(&self, endpoint: &str) -> String {
        format!("{}{}", self.base_url, endpoint)
    }
}
