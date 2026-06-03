use std::path::PathBuf;

use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Generation {
    pub id: i64,
    pub number: u32,
    pub created_at: i64,
    pub description: Option<String>,
    pub current: bool,
    pub packages: Vec<Package>,
}

impl Default for Generation {
    fn default() -> Self {
        Self {
            id: 0,
            number: 0,
            created_at: 0,
            description: None,
            current: false,
            packages: Vec::new(),
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Repo {
    pub id: Option<i64>,
    pub url: Option<String>,
    pub installed_at: Option<i64>,
    pub name: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub homepage: Option<String>,
    pub packages: Vec<Package>,
}

impl Default for Repo {
    fn default() -> Self {
        Self {
            id: None,
            url: None,
            installed_at: None,
            name: String::new(),
            description: None,
            icon: None,
            homepage: None,
            packages: Vec::new(),
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct Package {
    pub id: Option<i64>,
    pub is_installed: Option<bool>,
    pub is_cached: Option<bool>,
    pub is_disabled: Option<bool>,
    pub installed_at: Option<i64>,

    pub name: String,
    pub version: String,
    pub description: String,

    pub native_depiction: Option<String>,
    pub license: Option<String>,
    pub homepage: Option<String>,
    pub category: Option<String>,
    pub icon: Option<String>,

    pub platforms: Vec<String>,

    pub dependencies: Option<Vec<Dependency>>,
    pub source: Option<Source>,
    pub autobump: Option<Autobump>,
    pub scripts: Option<Scripts>,
}

impl Default for Package {
    fn default() -> Self {
        Self {
            id: None,
            is_installed: None,
            is_cached: None,
            is_disabled: None,
            installed_at: None,
            name: String::new(),
            version: String::new(),
            description: String::new(),
            native_depiction: None,
            license: None,
            homepage: None,
            category: None,
            icon: None,
            platforms: Vec::new(),
            dependencies: None,
            source: None,
            autobump: None,
            scripts: None,
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct Dependency {
    pub name: Option<String>,
    pub version: Option<String>,
    pub is_build: Option<bool>,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct Source {
    pub url: Vec<String>,
    pub sha256: String,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct Autobump {
    pub url: String,
    pub regex: String,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct Scripts {
    pub system: Option<String>,
    pub build: String,
    pub patches: Option<Vec<String>>,
}

impl Package {
    pub fn from_path(path: impl Into<PathBuf>) -> Result<Self, Error> {
        let content = std::fs::read_to_string(path.into())?;
        let config: Package = kdl::de::from_str(&content)?;
        Ok(config)
    }

    pub fn from_str(s: &str) -> Result<Self, Error> {
        let config: Package = kdl::de::from_str(s)?;
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const RECIPE: &str = r#"
    is_disabled #false 
    is_cached #true

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
"#;

    #[test]
    fn parse() -> miette::Result<()> {
        let config: Package = kdl::de::from_str(RECIPE)?;
        println!("{:#?}", config);
        Ok(())
    }
}
