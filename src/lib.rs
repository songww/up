pub mod archive;
pub mod dl;
pub mod ghapi;
pub mod opt;
pub mod sysinfo;
pub mod ui;

pub trait Anyhow<T> {
    fn anyhow(self) -> anyhow::Result<T>;
}

impl<T> Anyhow<T> for surf::Result<T> {
    fn anyhow(self) -> anyhow::Result<T> {
        self.map_err(|err| anyhow::anyhow!(err))
    }
}
