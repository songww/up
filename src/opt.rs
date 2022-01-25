use std::{path, str::FromStr};

use chrono::prelude::*;
use clap::{ArgEnum, Parser, Subcommand};
use serde::{Deserialize, Serialize};

use crate::ghapi;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
#[clap(global_setting(clap::AppSettings::PropagateVersion))]
#[clap(global_setting(clap::AppSettings::UseLongFormatForHelpSubcommand))]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Commands,
    #[clap(long, help = "Config.")]
    pub config: Option<path::PathBuf>,
}

#[non_exhaustive]
#[derive(Subcommand, Debug)]
pub enum Commands {
    Install {
        name: String,
        /// Which version want to install. The latest version will be selected, if not specified.
        version: Option<String>,
        #[clap(long, help = "Github repository, `{owner}/{name}`")]
        repo: String,
        #[clap(long, help = "Check for the latest release including prerelease")]
        latest: bool,
        // #[clap(long, help = "Which version will be installed.")]
        // version: bool,
        #[clap(long, help = "Which asset.")]
        asset_name: Option<String>,
        #[clap(long, help = "Allow pre-release.")]
        allow_prerelease: bool,
        #[clap(long, help = "Do something after downloaded.")]
        after_downloaded: Option<String>,
        #[clap(long, arg_enum, help = "Type, exe/font/config.")]
        r#type: Type,
        #[clap(long, help = "which to unpack")]
        unpack: Option<String>,
        #[clap(long, help = "unpack as")]
        binname: Option<String>,
    },
    Update {
        name: String,
        /// Which version want to install. The latest version will be selected, if not specified.
        version: Option<String>,
        #[clap(long, help = "Which asset.")]
        asset_name: Option<String>,
        #[clap(long, help = "Github repository, `{owner}/{name}`")]
        repo: Option<String>,
        #[clap(long, help = "Allow pre-release.")]
        allow_prerelease: bool,
        #[clap(long, help = "Do something after downloaded.")]
        after_downloaded: Option<String>,
    },

    Upgrade {
        #[clap(long, help = "Allow pre-release.")]
        allow_prerelease: bool,
    },
}

#[non_exhaustive]
#[derive(Clone, Debug, Deserialize, Serialize, ArgEnum)]
pub enum Type {
    Font,
    Executable,
    Configuration,
}

impl Default for Type {
    fn default() -> Self {
        Type::Executable
    }
}

impl FromStr for Type {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "exe" => Ok(Type::Executable),
            "font" => Ok(Type::Font),
            "config" => Ok(Type::Configuration),
            _ => Err(anyhow::anyhow!(
                "Invalid type `{}`, must be one of 'exe', 'font', 'config'.",
                s
            )),
        }
    }
}

#[derive(Debug, Default)]
pub struct Repo {
    name: String,
    owner: String,
}

impl Repo {
    pub fn new(owner: String, name: String) -> Repo {
        Repo { name, owner }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn owner(&self) -> &str {
        &self.owner
    }

    pub fn github(&self, gh: ghapi::Github) -> ghapi::GhRepo {
        gh.repo(self.owner.to_string(), self.name.to_string())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Options {
    pub name: String,
    pub repo: String,
    pub latest: bool,
    pub version: Option<String>,
    pub asset_name: Option<String>,
    pub allow_prerelease: bool,
    pub after_downloaded: Option<String>,
    pub r#type: Type,
    pub unpack: Option<String>,
    pub binname: Option<String>,
    pub app_version: Option<AppVersion>,
}

impl Options {
    pub fn repo(&self) -> anyhow::Result<Repo> {
        // anyhow::ensure!(
        //     self.repo.is_some(),
        //     "repo must in format `{{owner}}/{{name}}`"
        // );
        // let repo = repo.as_ref().unwrap();
        anyhow::ensure!(
            self.repo.contains("/"),
            "repo must in format `{{owner}}/{{name}}`"
        );
        let repo = self.repo.split("/").collect::<Vec<_>>();
        anyhow::ensure!(repo.len() == 2, "repo must in format `{{owner}}/{{name}}`");
        Ok(Repo::new(repo[0].to_string(), repo[1].to_string()))
    }
}

impl Into<AppConfig> for Options {
    fn into(self) -> AppConfig {
        AppConfig {
            name: self.name,
            repo: self.repo,
            asset_name: self.asset_name.unwrap(),
            allow_prerelease: self.allow_prerelease,
            after_downloaded: self.after_downloaded,
            r#type: self.r#type,
            unpack: self.unpack,
            binname: self.binname,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Config {
    pub apps: indexmap::IndexMap<String, AppConfig>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AppConfig {
    pub name: String,
    pub repo: String,
    pub asset_name: String,
    #[serde(default)]
    pub allow_prerelease: bool,
    pub after_downloaded: Option<String>,
    #[serde(default)]
    pub r#type: Type,
    pub unpack: Option<String>,
    pub binname: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AppVersion {
    pub name: String,
    pub version: String,
    pub files: Vec<std::path::PathBuf>,
    pub updated_at: DateTime<Local>,
    pub last_latest_at: Option<DateTime<Local>>,
    pub last_releases_at: Option<DateTime<Local>>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AppVersions {
    pub apps: indexmap::IndexMap<String, AppVersion>,
}
