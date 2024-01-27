use crate::config::Config;
use crate::pkg::{Asset, Assets, Release, Version};
use anyhow::{bail, Result};
use regex::Regex;
use reqwest::blocking::{Client, Request};
use reqwest::StatusCode;
use serde::Deserialize;

pub struct GitHubClient {
    base_url: String,
    client: Client,
    semver: Regex,
    date: Regex,
    auth: Option<(String, String)>,
}

impl GitHubClient {
    pub fn new(cfg: &Config) -> Self {
        let semver = Regex::new(r"(v\d{1,2}\.\d{1,2}\.\d{1,3})").unwrap();
        let date = Regex::new(r"^20\d\d-\d\d-\d\d$").unwrap();

        let auth = cfg
            .auth
            .as_ref()
            .map(|auth| (auth.client_id.clone(), auth.client_secret.clone()));

        Self {
            base_url: "https://api.github.com".to_string(),
            client: Client::new(),
            semver,
            date,
            auth,
        }
    }

    fn get_release(&self, url: String) -> Result<Option<Release>> {
        let req = self.build_request(&url, "application/json")?;
        let res = self.client.execute(req)?;

        let release = match res.status() {
            StatusCode::OK => {
                log::debug!("200 OK for GET {}", url);

                let body = res.text()?;
                log::debug!("Response body: {}", body);

                let body: GHRelease = serde_json::from_str(&body)?;
                body
            }
            StatusCode::NOT_FOUND => return Ok(None),
            s => {
                log::warn!("Unexpected status code for GET {}: {}", url, s);
                bail!("unexpected status code: {}", s)
            }
        };

        let tag_name = self.try_get_tag(&release.tag_name)?;
        log::debug!("Found tag in release: {}", tag_name);

        Ok(Some(Release {
            name: release.name,
            tag: tag_name,
            prerelease: release.prerelease,
            assets: release.assets,
        }))
    }

    fn try_get_tag(&self, tag: &str) -> Result<String> {
        // Try semver
        if let Some(matches) = self.semver.captures(tag) {
            if let Some(m) = matches.get(1) {
                return Ok(m.as_str().to_string());
            }
        }

        // Try date: yyyy-mm-dd
        if self.date.is_match(tag) {
            return Ok(tag.to_string());
        }

        Ok(tag.to_string())
    }

    fn build_request(&self, url: &str, mime: &str) -> Result<Request> {
        let req = self
            .client
            .get(url)
            .header("User-Agent", "dev-tool-installer")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .header("Accept", mime);

        let req = match &self.auth {
            Some((client_id, client_secret)) => req.basic_auth(client_id, Some(client_secret)),
            None => req,
        };

        let req = req.build()?;
        Ok(req)
    }

    pub fn latest(&self, repo: &str) -> Result<Option<Release>> {
        let repo = repo.trim_start_matches("https://github.com/");
        let url = format!("{}/repos/{}/releases/latest", self.base_url, repo);
        self.get_release(url)
    }

    pub fn get_from_tag(&self, repo: &str, tag: &str) -> Result<Option<Release>> {
        let repo = repo.trim_start_matches("https://github.com/");
        let url = format!("{}/repos/{}/releases/tags/{}", self.base_url, repo, tag);
        self.get_release(url)
    }

    pub fn try_get_release(&self, repo: &str, version: Option<Version>) -> Result<Option<Release>> {
        match &version {
            Some(v) => {
                let version = v.to_string();
                self.get_from_tag(repo, &version)
            }
            None => self.latest(repo),
        }
    }
}

#[derive(Clone, Deserialize)]
struct GHRelease {
    name: String,
    tag_name: String,
    prerelease: bool,
    assets: Vec<Asset>,
}

impl Assets for GitHubClient {
    fn download(&self, asset: &Asset) -> Result<Vec<u8>> {
        let req = self.build_request(&asset.url, "application/octet-stream")?;
        let res = self.client.execute(req)?;

        let status = res.status();
        if status != StatusCode::OK {
            log::warn!("Unexpected status code: GET {}: {}", asset.url, status);
            bail!("unexpected status code: GET {}: {}", asset.url, status);
        }

        let bytes = res.bytes()?;
        Ok(bytes.into())
    }
}
