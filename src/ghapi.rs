use std::str::FromStr;

use chrono::prelude::*;
use serde::{Deserialize, Serialize};

use crate::dl;

#[derive(Clone, Debug)]
pub struct Github {
    c: surf::Client,
}

#[allow(non_upper_case_globals)]
const GithubAccept: &str = "application/vnd.github.v3+json";

impl Default for Github {
    fn default() -> Self {
        Github::new("https://api.github.com/")
    }
}

impl Github {
    pub fn new(api: &str) -> Github {
        let api = surf::Url::parse(api).unwrap();
        let mime = surf::http::Mime::from_str(GithubAccept).unwrap();
        let mut accept = surf::http::content::Accept::new();
        accept.push(mime);
        let c = surf::Config::default()
            .set_timeout(std::time::Duration::from_secs(10).into())
            .set_base_url(api)
            .add_header(accept.name(), accept.value())
            .unwrap()
            .try_into()
            .unwrap();
        Github { c }
    }

    pub async fn get(&self, uri: &str) -> surf::Result<surf::Response> {
        self.c.get(uri).await
    }

    pub fn repo(&self, owner: String, name: String) -> GhRepo {
        GhRepo {
            gh: self.clone(),
            name,
            owner,
        }
    }
}

#[derive(Clone, Debug)]
pub struct GhRepo {
    gh: Github,
    name: String,
    owner: String,
}

impl GhRepo {
    pub fn releases(&self) -> GhRelease {
        GhRelease { repo: self.clone() }
    }

    fn path(&self) -> String {
        format!("/repos/{}/{}", self.owner, self.name)
    }
}

#[derive(Clone, Debug)]
pub struct GhRelease {
    repo: GhRepo,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Pagination {
    page: usize,
    per_page: usize,
}

impl Default for Pagination {
    fn default() -> Self {
        Pagination {
            page: 1,
            per_page: 20,
        }
    }
}

impl Pagination {
    pub fn new(page: usize, per_page: usize) -> Self {
        Pagination { page, per_page }
    }

    pub fn page(&self) -> usize {
        self.page
    }

    pub fn set_page(&mut self, page: usize) {
        self.page = page;
    }

    pub fn per_page(&mut self) -> usize {
        self.per_page
    }

    pub fn of_page(page: usize) -> Self {
        Pagination {
            page,
            ..Default::default()
        }
    }

    pub fn of_per_page(per_page: usize) -> Self {
        Pagination {
            per_page,
            ..Default::default()
        }
    }
}

pub fn pagination(page: usize, per_page: usize) -> Pagination {
    Pagination::new(page, per_page)
}

impl GhRelease {
    fn path(&self) -> String {
        "releases".into()
    }

    pub async fn releases(
        &self,
        page: Option<Pagination>,
        since: &mut Option<DateTime<Local>>,
    ) -> surf::Result<Vec<Release>> {
        let path = vec![self.repo.path(), self.path()].join("/");
        let mut request = self.repo.gh.c.get(path);
        if let Some(page) = page {
            request = request.query(&page)?;
        };
        if let Some(since) = since {
            let since = surf::http::conditional::IfModifiedSince::new(since.clone().into());
            request = request.header(since.name(), since.value());
        }
        let request = request.build();
        let response = surf::client().send(request).await?;
        let mut response = is_ok(response).await?;
        if let Some(last) = surf::http::conditional::LastModified::from_headers(&response)? {
            println!("last modified: {:?}", last.modified());
            since.replace(last.modified().into());
        };
        response
            .body_json()
            .await
            .and_then(|mut releases: Vec<Release>| {
                releases.iter_mut().for_each(|release| {
                    release.gh.replace(self.clone());
                });
                Ok(releases)
            })
    }

    pub async fn latest(&self, since: &mut Option<DateTime<Local>>) -> surf::Result<Release> {
        let repos = self.repo.path();
        let releases = self.path();
        let mut seg: Vec<&str> = Vec::with_capacity(5);
        seg.push(&repos);
        seg.push(&releases);
        seg.push("latest");
        let mut request = self.repo.gh.c.get(seg.join("/"));
        if let Some(since) = since {
            let since = surf::http::conditional::IfModifiedSince::new(since.clone().into());
            request = request.header(since.name(), since.value());
        }
        let mut response = is_ok(request.await?).await?;
        if let Some(last) = surf::http::conditional::LastModified::from_headers(&response)? {
            since.replace(last.modified().into());
        };
        response.body_json().await.map(|mut release: Release| {
            release.gh.replace(self.clone());
            release
        })
    }

    pub async fn oftag(
        &self,
        tagname: &str,
        since: &mut Option<DateTime<Local>>,
    ) -> surf::Result<Release> {
        let mut seg = Vec::with_capacity(5);
        let repos = self.repo.path();
        let releases = self.path();
        seg.push("");
        seg.push(&repos);
        seg.push(&releases);
        seg.push("tags");
        seg.push(tagname);
        let mut request = self.repo.gh.c.get(seg.join("/"));
        if let Some(since) = since {
            let since = surf::http::conditional::IfModifiedSince::new(since.clone().into());
            request = request.header(since.name(), since.value());
        }
        let mut response = is_ok(request.await?).await?;
        if let Some(last) = surf::http::conditional::LastModified::from_headers(&response)? {
            since.replace(last.modified().into());
        };
        response.body_json().await.map(|mut release: Release| {
            release.gh.replace(self.clone());
            release
        })
    }

    async fn assets(
        &self,
        release: usize,
        page: Option<Pagination>,
        since: &mut Option<DateTime<Local>>,
    ) -> surf::Result<Vec<Asset>> {
        let mut seg: Vec<&str> = Vec::with_capacity(5);
        let repos = self.repo.path();
        let releases = self.path();
        let release = release.to_string();
        seg.push(&repos);
        seg.push(&releases);
        seg.push(&release);
        seg.push("assets");
        let mut request = self.repo.gh.c.get(seg.join("/"));
        if let Some(page) = page {
            request = request.query(&page)?;
        }
        if let Some(since) = since {
            let since = surf::http::conditional::IfModifiedSince::new(since.clone().into());
            request = request.header(since.name(), since.value());
        }
        let mut response = is_ok(request.await?).await?;
        if let Some(last) = surf::http::conditional::LastModified::from_headers(&response)? {
            since.replace(last.modified().into());
        };
        response.body_json().await
    }
}

pub async fn is_ok(mut resp: surf::Response) -> surf::Result<surf::Response> {
    let code = resp.status();
    if code.is_redirection() || code.is_client_error() || code.is_server_error() {
        let err = surf::Error::from_str(code, resp.body_string().await?);
        Err(err)
    } else {
        Ok(resp)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Release {
    id: usize,
    url: surf::Url,
    pub name: String,
    #[serde(rename(deserialize = "body"))]
    desc: Option<String>,
    #[serde(rename(deserialize = "tag_name"))]
    pub tagname: Option<String>,
    prerelease: bool,
    pub published_at: DateTime<Local>,
    #[serde(skip)]
    gh: Option<GhRelease>,
}

impl Release {
    pub async fn assets(&self, since: &mut Option<DateTime<Local>>) -> surf::Result<Vec<Asset>> {
        let gh = self.gh.clone().unwrap();
        gh.assets(self.id, Pagination::default().into(), since)
            .await
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Asset {
    pub name: String,
    pub url: surf::Url,
    pub size: usize,
    pub label: Option<String>,
    pub updated_at: DateTime<Local>,
    pub browser_download_url: surf::Url,
}

impl Asset {
    // Download the release
    pub async fn download(&self) -> anyhow::Result<std::path::PathBuf> {
        println!(
            "Downloading {} from {}",
            self.name, &self.browser_download_url
        );
        let tempdir = std::env::temp_dir().join("up");
        // let path = tempdir.join(&self.name);
        let downloader = dl::Downloader::new(&tempdir);
        downloader
            .download(&self.browser_download_url, &self.name)
            .await?;

        Ok(tempdir.join(&self.name))
    }
}

impl Release {
    pub fn url(&self) -> &surf::Url {
        &self.url
    }
}

impl ToString for Release {
    fn to_string(&self) -> String {
        format!(
            "{}\t{}\t({})",
            self.name,
            self.desc.clone().unwrap_or_default(),
            self.published_at.format("%F %T")
        )
    }
}

impl ToString for Asset {
    fn to_string(&self) -> String {
        let label = if let Some(label) = &self.label {
            if label.len() > 0 {
                format!("({})", label)
            } else {
                String::new()
            }
        } else {
            String::new()
        };
        format!(
            "{}\t{}\t\t{}",
            bytesize::ByteSize::b(self.size as _).to_string_as(true),
            self.name,
            &label
        )
    }
}

// #[derive(Error, Debug)]
// pub enum GithubError {
//     #[error("Repo Not Found")]
//     NotFound(StatusCode),
// }

/*
impl RepoInfo {
    pub async fn from_url(url: &str) -> GBDResult<Self> {
        let mut url = url.to_string();
        if !url.contains("https://") && !url.contains("http://") {
            url = format!("https://{}", url);
        }
        if !url.contains("github") {
            return Err(Box::new(GithubError::NotFound(StatusCode::NOT_IMPLEMENTED)));
        }
        let resp = reqwest::get(&url).await?;
        if resp.status() == StatusCode::OK {
            let path = resp.url().path();
            let repoinfo_vec: Vec<&str> = path.split('/').collect();
            let releases_api_url = format!(
                "https://api.github.com/repos/{}/{}/releases",
                repoinfo_vec[1].to_string(),
                repoinfo_vec[2].to_string()
            );
            Ok(RepoInfo {
                user_name: repoinfo_vec[1].to_string(),
                repo_name: repoinfo_vec[2].to_string(),
                url,
                releases_api_url,
                ..Default::default()
            })
        } else {
            Err(Box::new(GithubError::NotFound(resp.status())))
        }
    }

    // Fetch the latest release from Github including Pre-release
    pub async fn get_latest_release(&mut self) -> GBDResult<()> {
        let client = reqwest::Client::builder()
            .user_agent("github-bin-downloader")
            .build()?;
        let resp = client
            .get(&self.releases_api_url)
            .send()
            .await?
            .text()
            .await?;
        let repo: Value = serde_json::from_str(&resp)?;
        let length = repo[0]["assets"]
            .as_array()
            .expect("Cannot convert to Array")
            .len();
        let mut releases: Vec<Release> = Vec::new();
        for i in 0..length {
            releases.push(Release {
                name: utils::sanitize_str_to_string(&repo[0]["assets"][i]["name"]),
                url: utils::sanitize_str_to_string(&repo[0]["assets"][i]["browser_download_url"]),
            });
        }
        self.releases = releases;
        Ok(())
    }

    // Get all the latest stable releases from Github releases
    pub async fn get_latest_stable_release(&mut self) -> GBDResult<()> {
        let client = reqwest::Client::builder()
            .user_agent("github-bin-downloader")
            .build()?;
        let resp = client
            .get(&self.releases_api_url)
            .send()
            .await?
            .text()
            .await?;
        let repo: Value = serde_json::from_str(&resp)?;
        let length = repo.as_array().expect("Cannot convert to Array").len();
        let mut releases: Vec<Release> = Vec::new();
        for i in 0..length {
            if !repo[i]["prerelease"]
                .as_bool()
                .expect("Cannot convert to bool")
            {
                let length = repo[i]["assets"]
                    .as_array()
                    .expect("Cannot convert to Array")
                    .len();
                for j in 0..length {
                    releases.push(Release {
                        name: utils::sanitize_str_to_string(&repo[i]["assets"][j]["name"]),
                        url: utils::sanitize_str_to_string(
                            &repo[i]["assets"][j]["browser_download_url"],
                        ),
                    });
                }
                self.releases = releases;
                return Ok(());
            }
        }
        Ok(())
    }

    // Search the releases for the host OS
    pub async fn search_releases_for_os(&self) -> GBDResult<Vec<Release>> {
        let sys_info = sysinfo::SystemInfo::new();
        let mut releases: Vec<Release> = Vec::new();
        match sys_info.platform_os() {
            sysinfo::PlatformOS::Darwin => {
                sysinfo::APPLE.iter().for_each(|mac| {
                    self.releases.iter().for_each(|release| {
                        if release.name.to_lowercase().contains(mac) {
                            releases.push(release.clone());
                        }
                    });
                });
            }
            sysinfo::PlatformOS::Linux => {
                sysinfo::LINUX.iter().for_each(|linux| {
                    self.releases.iter().for_each(|release| {
                        if release.name.to_lowercase().contains(linux) {
                            releases.push(release.clone());
                        }
                    });
                });
            }
            _ => {}
        }
        Ok(releases)
    }

    // Search the releases for the host Arch
    pub async fn search_releases_for_arch(&self) -> GBDResult<Vec<Release>> {
        let sys_info = sysinfo::SystemInfo::new();
        let mut releases: Vec<Release> = Vec::new();
        match sys_info.platform_arch() {
            sysinfo::PlatformArch::X8664 => {
                sysinfo::AMD64.iter().for_each(|arch| {
                    self.releases.iter().for_each(|release| {
                        if release.name.contains(arch) {
                            releases.push(release.clone());
                        }
                    });
                });
            }
            sysinfo::PlatformArch::Arm64 => {
                sysinfo::ARM64.iter().for_each(|arch| {
                    self.releases.iter().for_each(|release| {
                        if release.name.contains(arch) {
                            releases.push(release.clone());
                        }
                    });
                });
            }
            _ => {}
        }
        Ok(releases)
    }
}
*/
