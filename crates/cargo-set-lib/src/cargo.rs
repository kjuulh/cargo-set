use std::collections::BTreeMap;
use std::path::PathBuf;

use anyhow::Context;
use cargo_toml::{Dependency, Manifest};

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
        let mut s = CargoManifest::new(path.clone(), manifest);

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
                let mut member_path = s.root_path.parent().unwrap().to_path_buf();
                member_path.push(member);
                member_path.push("Cargo.toml");

                let manifest = self.load_cargo(&member_path)?;
                members.insert(member_path, manifest);
            }

            if members.len() > 0 {
                s.members = Some(members);
            }
        }

        Ok(s)
    }

    fn update_version<'s>(
        &self,
        s: &'s mut CargoManifest,
        package: impl Into<String>,
        version: impl Into<String>,
    ) -> anyhow::Result<&'s mut CargoManifest> {
        let version = version.into();
        let package = package.into();

        // Update version in root manifest
        if let Some(pkg) = &s.root_manifest.package {
            if pkg.name == package {
                s.root_manifest
                    .package
                    .as_mut()
                    .map(|p| p.version.set(version.clone()));
            }
        } else {
            self.update_dependencies(&mut s.root_manifest.dependencies, &package, &version);
        }
        if let Some(workspace) = s.root_manifest.workspace.as_mut() {
            self.update_dependencies(&mut workspace.dependencies, &package, &version);
        }
        self.fs.write(
            &s.root_path,
            toml::to_string_pretty(&s.root_manifest)?
                .as_bytes()
                .to_vec(),
        )?;

        // If there are workspace members, update version in each of them
        if let Some(members) = &mut s.members {
            for (path, manifest) in members.iter_mut() {
                let member_path = path;

                if let Some(pkg) = &manifest.package {
                    if pkg.name == package {
                        manifest
                            .package
                            .as_mut()
                            .map(|p| p.version.set(version.clone()));
                    }
                } else {
                    self.update_dependencies(&mut manifest.dependencies, &package, &version);
                    self.fs.write(
                        &member_path,
                        toml::to_string_pretty(&manifest)?.as_bytes().to_vec(),
                    )?;
                }
            }
        }

        Ok(s)
    }

    fn update_dependencies(
        &self,
        dependencies: &mut BTreeMap<String, Dependency>,
        package: &String,
        version: &String,
    ) {
        for (_name, dep_version) in dependencies
            .iter_mut()
            .filter(|(name, _)| name.eq(&package))
        {
            match dep_version {
                Dependency::Simple(dep) => *dep = version.clone(),
                Dependency::Inherited(_) => {}
                Dependency::Detailed(dep) => dep.version = Some(version.clone()),
            }
        }
    }
}

#[cfg(test)]
mod test {
    use std::assert_eq;

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

    #[test]
    fn can_no_children() -> anyhow::Result<()> {
        let root_manifest_toml = b"name = 'root'\nversion = '0.1.0'\n workspace = { members = [] }";

        let root_manifest_path = PathBuf::from("Cargo.toml");

        let mut fs = MockFileSystem::new();
        fs.add_file(root_manifest_path.clone(), root_manifest_toml.to_vec());

        let cargo_manifest = CargoManifestService::new(fs)
            .load_manifest(&root_manifest_path)
            .unwrap();
        assert!(cargo_manifest.members.is_none());

        Ok(())
    }

    #[test]
    fn can_update_version() -> anyhow::Result<()> {
        let root_manifest_toml = r#"
            [workspace] 
            members = ['child', "other"]

            [workspace.dependencies]
            child = { path = "child", version = "0.2.0"}
            other = { path = "child", version = "0.2.0"}


            [package]
            name = 'root'
            version = '0.1.0'

            [dependencies]
            child.workspace = true
            "#;
        let child_manifest_toml = b"name = 'child'\nversion = '0.2.0'";
        let other_child_manifest_toml = b"name = 'other'\nversion = '0.1.0'";

        let root_manifest_path = PathBuf::from("Cargo.toml");
        let child_manifest_path = PathBuf::from("child/Cargo.toml");
        let other_child_manifest_path = PathBuf::from("other/Cargo.toml");

        let mut fs = MockFileSystem::new();
        fs.add_file(
            root_manifest_path.clone(),
            root_manifest_toml.as_bytes().to_vec(),
        );
        fs.add_file(child_manifest_path.clone(), child_manifest_toml.to_vec());
        fs.add_file(
            other_child_manifest_path.clone(),
            other_child_manifest_toml.to_vec(),
        );

        let cargo_manifest_service = CargoManifestService::new(fs);

        let mut cargo_manifest = cargo_manifest_service
            .load_manifest(&root_manifest_path)
            .unwrap();

        cargo_manifest_service.update_version(&mut cargo_manifest, "child", "0.3.0")?;

        match cargo_manifest
            .root_manifest
            .workspace
            .unwrap()
            .dependencies
            .get("child")
            .unwrap()
        {
            cargo_toml::Dependency::Simple(_) => todo!(),
            cargo_toml::Dependency::Inherited(_) => todo!(),
            cargo_toml::Dependency::Detailed(d) => assert_eq!(d.version.as_ref().unwrap(), "0.3.0"),
        }

        assert_eq!(
            cargo_manifest.root_manifest.package.unwrap().version(),
            "0.1.0"
        );

        assert_eq!(
            cargo_manifest
                .members
                .as_ref()
                .unwrap()
                .get(&PathBuf::from("child/Cargo.toml"))
                .unwrap()
                .package()
                .version(),
            "0.3.0"
        );

        assert_eq!(
            cargo_manifest
                .members
                .as_ref()
                .unwrap()
                .get(&PathBuf::from("other/Cargo.toml"))
                .unwrap()
                .package()
                .version(),
            "0.1.0"
        );

        Ok(())
    }
}
