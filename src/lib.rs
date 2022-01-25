use std::str::FromStr;

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

pub fn is_compatible(app: &str, version: &str, asset: &str) -> bool {
    let triple = take_triple(app, version, asset);
    let triple = target::Triple::from_str(triple).expect(triple);
    let host = target::HOST;
    host.vendor == triple.vendor
        && host.architecture == triple.architecture
        && host.operating_system == triple.operating_system
}

fn take_triple<'a>(app: &str, version: &str, asset: &'a str) -> &'a str {
    let mut name = asset;
    if asset.starts_with(app) {
        name = unsafe { name.get_unchecked(app.len()..) };
    }
    if name.starts_with("-") {
        name = unsafe { name.get_unchecked(1..) };
    }
    if name.starts_with(version) {
        name = unsafe { name.get_unchecked(version.len()..) };
    }
    if name.starts_with("v") && unsafe { name.get_unchecked(1..) }.starts_with(version) {
        name = unsafe { name.get_unchecked(version.len() + 1..) };
    }
    if name.starts_with("-") {
        name = unsafe { name.get_unchecked(1..) };
    }
    if let Some(pos) = name.find(|c| c == '.') {
        name = unsafe { name.get_unchecked(..pos) };
    }
    name
}
