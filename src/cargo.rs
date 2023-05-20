use std::path::PathBuf;

use anyhow::Context;
use cargo_toml::Manifest;

pub fn parse_cargo(path: PathBuf) -> anyhow::Result<Manifest> {
    let content = std::fs::read(path).context("failed to read Cargo.toml from path")?;

    let manifest = Manifest::from_slice(&content).context("failed to parse Cargo.toml")?;

    Ok(manifest)
}
