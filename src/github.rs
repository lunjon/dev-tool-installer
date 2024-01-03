use crate::{
    config::Config,
    pkg::{Asset, Assets, Release, Releases},
};
use anyhow::{bail, Result};
use regex::Regex;
use reqwest::{blocking::Client, StatusCode};
use serde::Deserialize;

pub struct GitHubClient {
    repo: String,
    base_url: String,
    client: Client,
    semver: Regex,
    date: Regex,
    auth: Option<(String, String)>,
}

impl GitHubClient {
    pub fn new(cfg: &Config, repo: String) -> Self {
        let semver = Regex::new(r"(v\d{1,2}\.\d{1,2}\.\d{1,3})").unwrap();
        let date = Regex::new(r"^20\d\d-\d\d-\d\d$").unwrap();

        let auth = cfg
            .auth
            .as_ref()
            .map(|auth| (auth.client_id.clone(), auth.client_secret.clone()));

        Self {
            repo,
            base_url: "https://api.github.com".to_string(),
            client: Client::new(),
            semver,
            date,
            auth,
        }
    }

    fn get_release(&self, url: String) -> Result<Option<Release>> {
        let req = self
            .client
            .get(url)
            .header("User-Agent", "code-tools-cli")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .header("Accept", "application/json");

        let req = match &self.auth {
            Some((client_id, client_secret)) => req.basic_auth(client_id, Some(client_secret)),
            None => req,
        };

        let res = self.client.execute(req.build()?)?;
        let release = match res.status() {
            StatusCode::OK => {
                let body = res.text()?;
                let body: GHRelease = serde_json::from_str(&body)?;
                body
            }
            StatusCode::NOT_FOUND => return Ok(None),
            s => bail!("unexpected status code: {}", s),
        };

        let tag_name = self.try_get_tag(&release.tag_name)?;

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
}

impl Releases for GitHubClient {
    fn latest(&self) -> Result<Option<Release>> {
        let url = format!("{}/repos/{}/releases/latest", self.base_url, self.repo);
        self.get_release(url)
    }

    fn get_from_tag(&self, tag: &str) -> Result<Option<Release>> {
        let url = format!(
            "{}/repos/{}/releases/tags/{}",
            self.base_url, self.repo, tag
        );
        self.get_release(url)
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
        let req = self
            .client
            .get(asset.url.to_string())
            .header("User-Agent", "code-tools-cli")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .header("Accept", "application/octet-stream")
            .build()?;

        let res = self.client.execute(req)?;

        let status = res.status();
        if status != StatusCode::OK {
            bail!("unexpected status code: GET {}: {}", asset.url, status);
        }

        let bytes = res.bytes()?;
        Ok(bytes.into())
    }
}
