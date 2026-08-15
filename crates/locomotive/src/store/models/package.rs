use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Default)]
pub struct Package {
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
    pub sources: Option<Vec<String>>,
    pub sha256: Option<String>,
    pub system: Option<String>,
    pub build: Option<String>,
    pub patches: Option<Vec<String>>,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Default)]
pub struct Dependency {
    pub name: Option<String>,
    pub version: Option<String>,
    pub is_build: Option<bool>,
    pub path: Option<String>, // ??
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

    sources \
        "https://www.nano-editor.org/dist/v8/nano-8.7.1.tar.xz" \
        "https://www.mirror-nano-editor.org/dist/v8/nano-8.7.1.tar.xz"
    sha256 "76f0dcb248f2e2f1251d4ecd20fd30fb400a360a3a37c6c340e0a52c2d1cdedf"

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
"#;

    #[test]
    fn parse() -> miette::Result<()> {
        let config: Package = kdl::de::from_str(RECIPE)?;
        println!("{:#?}", config);
        Ok(())
    }
}
