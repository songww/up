use clap::Parser;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use up::{ghapi, opt, ui, Anyhow};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = opt::Cli::parse();

    let proj = directories::ProjectDirs::from("me", "songww", "up").unwrap();
    let config_path = if let Some(config) = cli.config {
        config
    } else {
        proj.config_dir().join("up.toml")
    };

    let mut configs = String::new();
    if !config_path.exists() {
        tokio::fs::create_dir(proj.config_dir()).await.ok();
        let src = toml::ser::to_string_pretty(&opt::Config {
            apps: indexmap::IndexMap::new(),
        })?;
        tokio::fs::File::create(&config_path)
            .await?
            .write(src.as_bytes())
            .await?;
    }
    tokio::fs::File::open(&config_path)
        .await?
        .read_to_string(&mut configs)
        .await?;
    let mut config: opt::Config = toml::de::from_str(&configs)?;

    let mut locked_versions = String::new();
    let locked_versions_path = proj.data_dir().join("versions.lock.toml");
    if !locked_versions_path.exists() {
        tokio::fs::create_dir(&proj.data_dir()).await.ok();
        let src = toml::ser::to_string_pretty(&opt::AppVersions {
            apps: indexmap::IndexMap::new(),
        })?;
        println!("default locked versions: {}", &src);
        tokio::fs::File::create(&locked_versions_path)
            .await?
            .write(src.as_bytes())
            .await?;
    }
    tokio::fs::File::open(&locked_versions_path)
        .await?
        .read_to_string(&mut locked_versions)
        .await?;
    // println!(
    //     "{} read_to_string {}",
    //     &locked_versions_path.display(),
    //     &locked_versions
    // );
    let mut locked_versions: opt::AppVersions = toml::de::from_str(&locked_versions)?;

    match cli.command {
        opt::Commands::Install {
            name,
            repo,
            unpack,
            binname,
            latest,
            version,
            asset_name,
            allow_prerelease,
            after_downloaded,
            r#type,
        } => {
            // anyhow::ensure!(
            //     locked_versions.apps.contains_key(&name),
            //     anyhow::anyhow!("{} already installed.", &name)
            // );
            let mut opts = opt::Options {
                name: name.to_string(),
                repo,
                latest,
                allow_prerelease,
                after_downloaded,
                asset_name,
                version,
                r#type,
                unpack,
                binname,
                app_version: None,
            };

            up(&mut opts).await?;
            locked_versions
                .apps
                .insert(name.to_string(), opts.app_version.clone().unwrap());
            config.apps.insert(name, opts.into());
        }
        opt::Commands::Update {
            name,
            mut repo,
            version,
            mut asset_name,
            mut allow_prerelease,
            mut after_downloaded,
        } => {
            let app = config
                .apps
                .get_mut(&name)
                .ok_or_else(|| anyhow::anyhow!("{} not installed yet.", &name))?;
            let cfg: opt::AppConfig = app.clone().try_into().unwrap();
            if repo.is_none() {
                repo.replace(cfg.repo);
            } else {
                app.repo = repo.as_ref().unwrap().clone().into();
            }
            if asset_name.is_none() {
                asset_name.replace(cfg.asset_name);
            } else {
                app.asset_name = asset_name.as_ref().unwrap().clone().into();
            }
            if allow_prerelease || cfg.allow_prerelease {
                allow_prerelease = true;
            }
            if after_downloaded.is_none() {
                after_downloaded = cfg.after_downloaded;
            }
            let app_version = locked_versions
                .apps
                .get(&name)
                .ok_or_else(|| anyhow::anyhow!("{} not installed yet.", &name))?
                .clone();
            let mut opts = opt::Options {
                name: name.to_string(),
                repo: repo.unwrap(),
                latest: version.is_none(),
                version,
                asset_name,
                allow_prerelease,
                after_downloaded,
                r#type: cfg.r#type,
                unpack: cfg.unpack,
                binname: cfg.binname,
                app_version: app_version.into(),
            };
            up(&mut opts).await?;
            locked_versions
                .apps
                .insert(name.to_string(), opts.app_version.clone().unwrap());
            config.apps.insert(name, opts.into());
        }

        opt::Commands::Upgrade { allow_prerelease } => {
            //
            for (name, appcfg) in config.apps.iter() {
                // up(&mut opt).await?;
            }
            unimplemented!()
        }

        _ => {
            unreachable!()
        }
    };

    // if opt.latest {
    //     repo.releases().await?;
    //     if opt.list {
    //         cli::display_all_options(&repo.releases)
    //             .await?
    //             .download_release()
    //             .await?;
    //         return Ok(());
    //     }
    //     match utils::compare_two_vector(
    //         &repo.search_releases_for_os().await?,
    //         &repo.search_releases_for_arch().await?,
    //     ) {
    //         Some(releases) => {
    //             cli::display_all_options(&releases).await?
    //             .download_release().await?;
    //             return Ok(());
    //         },
    //         None => show_error!("Cannot find a release for your OS and Arch\n Use --list flag to list all available options"),
    //     }
    // } else {
    //     repo.get_latest_stable_release().await?;
    //     if opt.list {
    //         cli::display_all_options(&repo.releases)
    //             .await?
    //             .download_release()
    //             .await?;
    //         return Ok(());
    //     }
    //     match utils::compare_two_vector(
    //         &repo.search_releases_for_os().await?,
    //         &repo.search_releases_for_arch().await?,
    //     ) {
    //         Some(releases) => {
    //             cli::display_all_options(&releases).await?
    //             .download_release().await?;
    //             return Ok(());
    //         },
    //         None => show_error!("Cannot find a release for your OS and Arch\n Use --list flag to list all available options"),
    //     }
    // }
    tokio::fs::write(&config_path, toml::ser::to_string_pretty(&config)?).await?;
    println!("locked versions: {:?}", &locked_versions);
    tokio::fs::write(
        &locked_versions_path,
        toml::ser::to_string_pretty(&locked_versions)?,
    )
    .await?;
    Ok(())
}

async fn up(opts: &mut opt::Options) -> anyhow::Result<()> {
    let gh = ghapi::Github::default();
    let repo = opts.repo()?.github(gh.clone());

    let basedir = directories::BaseDirs::new().unwrap();
    let executable_dir = basedir.executable_dir().unwrap();
    // check if downgrade
    // if let Some(app_version) = opts.app_version {
    //     opts.version
    // }

    let mut last_releases_since = opts.app_version.as_mut().and_then(|v| v.last_releases_at);
    let mut last_latest_since = opts.app_version.as_mut().and_then(|v| v.last_latest_at);
    let releases = repo.releases();
    let release = if let Some(version) = &opts.version {
        releases.oftag(version, &mut None).await.anyhow()?
    } else if !opts.latest {
        // 列出releases, 从中选择一个
        let options = releases
            .releases(None, &mut last_releases_since)
            .await
            .anyhow()?;
        ui::choose(&options, "Select the release").await?.clone()
    } else {
        let latest_release = releases.latest(&mut last_latest_since).await.anyhow()?;
        println!("Select the latest release: {}", latest_release.to_string());
        latest_release
    };
    if let Some(desc) = release.desc() {
        println!("");
        println!("{}", desc);
        println!("");
    }
    let assets = release.assets(&mut None).await.anyhow()?;
    let mut asset = None;
    if let Some(asset_name) = &opts.asset_name {
        assets
            .iter()
            .find(|asset| &asset.name == asset_name)
            .and_then(|asset_| asset.replace(asset_));
    }
    let asset = if asset.is_none() {
        let options: Vec<_> = assets
            .iter()
            .filter(|asset| up::is_compatible(&opts.name, &release.name, &asset.name))
            .map(|asset| asset.clone())
            .collect();
        if options.len() == 1 {
            let asset = &options[0];
            println!("Select the asset: {}", asset.to_string());
            asset.clone()
        } else if !options.is_empty() {
            ui::choose(&options, "Select the asset").await?.clone()
        } else {
            ui::choose(&assets, "Select the asset").await?.clone()
        }
    } else {
        asset.unwrap().clone()
    };

    opts.asset_name.replace(asset.name.clone());

    let executable = executable_dir.join(opts.binname.as_ref().unwrap_or(&opts.name));

    let asset_path = asset.download().await?;

    tokio::fs::remove_file(&executable).await?;
    let mut target = tokio::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .mode(0o755)
        .open(&executable)
        .await?;

    let ar_files = compressed::list_archive_files(std::fs::File::open(&asset_path).unwrap())?;

    if !ar_files.is_empty() {
        let mut source = tokio::fs::File::open(&asset_path).await?;
        if ar_files.len() == 1 && &ar_files[0] == "data" {
            compressed::tokio_support::uncompress_archive_file(&mut source, &mut target, "data")
                .await?;
        } else if let Some(unpack) = &opts.unpack.as_ref() {
            compressed::tokio_support::uncompress_archive_file(&mut source, &mut target, unpack)
                .await?;
        } else if ar_files.len() == 1 {
            compressed::tokio_support::uncompress_archive_file(
                &mut source,
                &mut target,
                &ar_files[0],
            )
            .await?;
        } else {
            anyhow::bail!("too many files in the asset archive, not support yet.");
        }
    } else {
        tokio::fs::copy(&asset_path, &executable).await?;
    }

    tokio::fs::remove_file(&asset_path).await?;

    opts.app_version.replace(opt::AppVersion {
        name: opts.name.to_string(),
        version: release.name,
        files: vec![executable.to_path_buf()],
        updated_at: asset.updated_at,
        last_latest_at: last_latest_since,
        last_releases_at: last_releases_since,
    });
    Ok(())
}
