use futures_lite::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use tokio::io::AsyncWriteExt;

pub struct Downloader {
    dir: std::path::PathBuf,
}

impl Downloader {
    pub fn new(dir: impl Into<std::path::PathBuf>) -> Downloader {
        let dir = dir.into();
        std::fs::create_dir_all(&dir).ok();
        Downloader { dir }
    }

    pub async fn download(
        &self,
        url: impl AsRef<str>,
        name: impl AsRef<str>,
    ) -> anyhow::Result<()> {
        let path = self.dir.join(name.as_ref());
        let mut f = tokio::fs::File::create(&path).await?;

        let attach = |err: reqwest::Error| -> anyhow::Error {
            anyhow::anyhow!(err).context(format!(
                "Can not download '{}' from `{}`",
                name.as_ref(),
                url.as_ref()
            ))
        };

        let timeout = std::time::Duration::from_secs(10);

        let cli = reqwest::Client::builder()
            .timeout(timeout * 6)
            .connect_timeout(timeout)
            .connection_verbose(true)
            .user_agent("Mozilla/5.0 (X11; Linux x86_64; rv:96.0) Gecko/20100101 Firefox/96.0")
            .redirect(reqwest::redirect::Policy::limited(5))
            .build()
            .map_err(attach)?;

        let resp = cli
            .get(url.as_ref())
            .send()
            .await
            .map_err(attach)?
            .error_for_status()
            .map_err(attach)?;
        // let mut downloaded = 0;
        let content_length = resp
            .content_length()
            .ok_or_else(|| anyhow::anyhow!("Cannot determine size of the content!"))?;
        let pb = ProgressBar::new(content_length);
        pb.set_style(ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})")
            .progress_chars("#>-"));
        let mut stream = resp.bytes_stream();
        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            let size = chunk.len();
            pb.inc(size as _);
            f.write_all(&chunk).await?;
        }
        pb.finish_with_message("Downloaded");
        Ok(())
    }
}
