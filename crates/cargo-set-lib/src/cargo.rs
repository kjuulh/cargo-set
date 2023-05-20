use std::collections::BTreeMap;
use std::path::PathBuf;

use anyhow::Context;
use cargo_toml::Manifest;

use crate::filesystem::FileSystem;

pub struct CargoManifestService<F: FileSystem> {
    fs: F,
}

#[derive(Debug, Clone)]
pub struct CargoManifest {
    root_path: PathBuf,
    root_manifest: Manifest,
    members: Option<BTreeMap<PathBuf, Manifest>>,
}

impl CargoManifest {
    pub fn new(root_path: PathBuf, root_manifest: Manifest) -> Self {
        Self {
            root_path,
            root_manifest,
            members: None,
        }
    }
}

impl<F: FileSystem> CargoManifestService<F> {
    pub fn new(fs: F) -> Self {
        Self { fs }
    }

    pub fn load_manifest(&self, path: &PathBuf) -> anyhow::Result<CargoManifest> {
        let manifest = self.load_cargo(path)?;
        let mut s = CargoManifest::new(
            path.parent()
                .map(|p| p.to_path_buf())
                .unwrap_or(PathBuf::from("/")),
            manifest,
        );

        let s = self.load_children(&mut s)?;

        Ok(s.to_owned())
    }

    fn load_cargo(&self, path: &PathBuf) -> anyhow::Result<Manifest> {
        let content = self
            .fs
            .read(path)
            .context("failed to read Cargo.toml from path")?;

        let manifest = Manifest::from_slice(&content).context("failed to parse Cargo.toml")?;

        Ok(manifest)
    }

    fn load_children<'s>(&self, s: &'s mut CargoManifest) -> anyhow::Result<&'s mut CargoManifest> {
        if let Some(workspace) = &s.root_manifest.workspace {
            let mut members = BTreeMap::new();

            for member in &workspace.members {
                let mut member_path = s.root_path.clone();
                member_path.push(member);
                member_path.push("Cargo.toml");

                let manifest = self.load_cargo(&member_path)?;
                members.insert(member_path, manifest);
            }
            s.members = Some(members);
        }

        Ok(s)
    }
}

#[cfg(test)]
mod test {
    use crate::filesystem::MockFileSystem;

    use super::*;

    #[test]
    fn can_load_children() -> anyhow::Result<()> {
        let root_manifest_toml =
            b"name = 'root'\nversion = '0.1.0'\n workspace = { members = ['child'] }";
        let child_manifest_toml = b"name = 'child'\nversion = '0.2.0'";

        let root_manifest_path = PathBuf::from("Cargo.toml");
        let child_manifest_path = PathBuf::from("child/Cargo.toml");

        let mut fs = MockFileSystem::new();
        fs.add_file(root_manifest_path.clone(), root_manifest_toml.to_vec());
        fs.add_file(child_manifest_path.clone(), child_manifest_toml.to_vec());

        let cargo_manifest = CargoManifestService::new(fs)
            .load_manifest(&root_manifest_path)
            .unwrap();
        let members = cargo_manifest.members.as_ref().unwrap();

        assert_eq!(1, members.len());
        assert!(members.contains_key(&child_manifest_path));

        Ok(())
    }
}
