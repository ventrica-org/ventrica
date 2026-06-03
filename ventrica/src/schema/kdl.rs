use std::path::PathBuf;

use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};

#[allow(dead_code)]
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Repo {
    /// The name of the repository.
    pub name: String,
    /// The description of the repository.
    pub description: String,
    /// The URL to the icon of the repository.
    pub icon: Option<String>,
    /// The homepage of the repository.
    pub homepage: Option<String>,
    /// The packages in the repository.
    pub packages: Vec<Package>,
}

#[allow(dead_code)]
#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct Root {
    package: Package,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Package {
    /// If package is installed.
    pub is_installed: Option<bool>,
    /// If package should be cached.
    pub is_cached: Option<bool>,
    /// If package is disabled.
    pub is_disabled: Option<bool>,
    /// Hash of the package.
    pub package_hash: Option<String>,
    /// The name of the package.
    pub name: String,
    /// The version of the package.
    pub version: String,
    /// The description of the package.
    pub description: String,
    /// The URL to the native depiction of the package.
    pub native_depiction: Option<String>,
    /// The license of the package.
    pub license: Option<String>,
    /// The homepage of the package.
    pub homepage: Option<String>,
    /// The category of the package.
    pub category: Option<String>,
    /// The URL to the icon of the package.
    pub icon: Option<String>,
    /// The platforms supported by the package.
    pub platforms: Vec<String>,
    /// The dependencies of the package.
    pub dependencies: Option<Dependencies>,
    /// The source information of the package.
    pub source: Option<Source>,
    /// The autobump configuration of the package.
    pub(crate) autobump: Option<Autobump>,
    /// The scripts for building the package.
    pub scripts: Option<Scripts>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Dependency {
    /// The name of the dependency.
    pub name: String,
    /// The version of the dependency.
    pub version: Option<String>,
    /// If the dependency is a build dependency.
    pub is_build: Option<bool>,
    /// The hash of the dependency package.
    pub(crate) package_hash: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Dependencies {
    /// The dependencies of the package.
    pub dep: Vec<Dependency>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Source {
    /// The URLs to the source of the package.
    url: Vec<String>,
    /// The SHA256 hash of the source archive.
    sha256: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub(crate) struct Autobump {
    /// The URL to check for new versions of the package.
    pub(crate) url: String,
    /// The regex to extract the version number from the URL.
    pub(crate) regex: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Scripts {
    /// The build script for the package.
    pub system: Option<String>,
    /// The build script for the package.
    pub build: String,
    /// The patches for the package.
    pub patches: Option<Vec<String>>,
}

impl Package {
    pub fn from_path(path: impl Into<PathBuf>) -> Result<Self, Error> {
        let content = std::fs::read_to_string(path.into())?;
        let config: Root = kdl::de::from_str(&content)?;
        Ok(config.package)
    }

    pub fn from_str(s: &str) -> Result<Self, Error> {
        let config: Root = kdl::de::from_str(s)?;
        Ok(config.package)
    }
}

#[cfg(test)]
mod tests {
    const RECIPE: &str = r#"package is_disabled=#false is_cached=#true {
    name "nano"
    version "8.7.1"
    description "Small, friendly text editor inspired by Pico"
    native_depiction "https://example.com/example.json"
    license "GPL-3.0-or-later"
    homepage "https://www.nano-editor.org/"
    category "editors"
    icon "https://github.com/claration/Impactor/blob/main/package/linux/icons/hicolor/64x64/apps/dev.khcrysalis.PlumeImpactor.png?raw=true"
    platforms "mac_arm64" "mac_x86-64" "linux_arm64" "linux_x86-64"

    dependencies {
        dep name="gettext" version="^1.0"
        dep name="ncurses"
        dep name="pkgconf" version="^1.0" is_build=#true
    }

    source {
        url \
        "https://www.nano-editor.org/dist/v8/nano-8.7.1.tar.xz" \
        "https://www.mirror-nano-editor.org/dist/v8/nano-8.7.1.tar.xz"
        sha256 "76f0dcb248f2e2f1251d4ecd20fd30fb400a360a3a37c6c340e0a52c2d1cdedf"
    }

    scripts {
        build """
        ./configure \
            --prefix=${PREFIX} \
            --enable-color \
            --enable-extra \
            --enable-multibuffer \
            --enable-nanorc \
            --enable-utf8 \
            --sysconfdir=${PREFIX}/etc
        make
        make install DESTDIR=${DESTDIR}
        """
    }
}"#;

    use super::*;

    #[test]
    fn parse() -> miette::Result<()> {
        let config: Root = kdl::de::from_str(RECIPE)?;
        println!("{:#?}", &config);
        println!(
            "{}",
            serde_json::to_string_pretty(&config).unwrap_or_default()
        );
        Ok(())
    }
}
