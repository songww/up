use dialoguer::Select;

use crate::ghapi;

pub trait Choosable: ToString {}

impl Choosable for ghapi::Release {}

impl Choosable for ghapi::Asset {}

pub async fn choose<T: Choosable>(options: &[T], desc: impl AsRef<str>) -> anyhow::Result<&T> {
    if options.is_empty() {
        // show_error!("No releases available!");
        // std::process::exit(1)
        return Err(anyhow::anyhow!("No choices available!"));
    }
    // println!("Select the release you want to download!");
    // println!("{}", desc.as_ref());
    let chosen = Select::new()
        .with_prompt(desc.as_ref())
        .items(&options)
        .default(0)
        .interact_opt()?
        .ok_or_else(|| anyhow::anyhow!("cancelled."))?;
    // println!("> {}", options[chosen].to_string());
    Ok(&options[chosen])
}
