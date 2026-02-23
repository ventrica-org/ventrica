//! Package recipe schema - parsed from YAML.
//!
//! # Example recipe
//!
//! ```yaml
//! meta:
//!   name: "openssl"
//!   version: "3.5.0"
//!   description: "TLS/SSL toolkit"
//!   license: "Apache-2.0"
//!   homepage: "https://openssl.org"
//!
//! deps:
//!   build:
//!     - "make"
//!     - "clang"
//!   run:
//!     - "zlib"
//!
//! src:
//!   fetch:
//!     url: "https://openssl.org/source/openssl-3.5.0.tar.gz"
//!     sha256: "abc123..."
//!
//! build:
//!   env:
//!     DESTDIR: ""
//!   system: autotools
//!   configure_args:
//!     - "--with-zlib"
//!   parallel: true
//!   post_install: "scripts/post_install.sh"
//! ```

use std::collections::HashMap;
use std::path::Path;

use serde::{Deserialize, Serialize};
use yaml_rust2::{Yaml, YamlLoader};

use crate::error::{Error, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Package {
    pub meta: Meta,
    pub deps: Deps,
    pub src: Source,
    pub build: Option<Build>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub store_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub filename: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub var_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recipe: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recipe_content: Option<String>,
}

/// Package identity and human-readable metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Meta {
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub category: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub native_description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub homepage: Option<String>,
    #[serde(default)]
    pub arch: Vec<String>,
    #[serde(default)]
    pub broken: bool,
    /// When true the built package is not packed into a `.var` archive and
    /// is not included in the repository manifest.  Use this for proprietary
    /// app bundles whose licences prohibit redistribution.
    #[serde(default)]
    pub no_cache: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Deps {
    #[serde(default)]
    pub build: Vec<String>,
    #[serde(default)]
    pub run: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Source {
    Fetch(FetchSource),
    Git(GitSource),
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchSource {
    pub url: String,
    pub sha256: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    #[serde(default)]
    pub mirrors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitSource {
    pub url: String,
    pub git_ref: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Build {
    #[serde(default)]
    pub env: HashMap<String, String>,
    pub system: BuildSystem,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run: Option<String>,
    #[serde(default)]
    pub patches: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BuildSystem {
    Shell,
    None,
    Application,
}

impl std::str::FromStr for BuildSystem {
    type Err = ();

    fn from_str(s: &str) -> std::result::Result<Self, ()> {
        Ok(match s {
            "shell" | "script" => Self::Shell,
            "none" => Self::None,
            "application" | "app" | "dmg" => Self::Application,
            _ => return Err(()),
        })
    }
}

impl std::fmt::Display for BuildSystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Shell => "shell",
            Self::None => "none",
            Self::Application => "application",
        })
    }
}

impl crate::schema::FromYaml for Package {
    fn from_file(path: &Path) -> Result<Self> {
        let text = std::fs::read_to_string(path).map_err(|e| Error::path(path, e.to_string()))?;
        Self::from_str_content(&text)
    }

    fn from_str_content(text: &str) -> Result<Self> {
        let docs = YamlLoader::load_from_str(text).map_err(|e| Error::YamlParse(e.to_string()))?;

        let doc = docs
            .into_iter()
            .next()
            .ok_or_else(|| Error::YamlParse("empty YAML document".into()))?;

        let meta = parse_meta(&doc)?;
        let deps = parse_deps(&doc)?;
        let src = parse_src(&doc)?;
        let build = parse_build(&doc)?;

        Ok(Self {
            meta,
            deps,
            src,
            build,
            store_name: None,
            filename: None,
            var_hash: None,
            recipe: None,
            recipe_content: None,
        })
    }
}

fn parse_meta(doc: &Yaml) -> Result<Meta> {
    let m = &doc["meta"];
    if m.is_badvalue() {
        return Err(Error::InvalidSchema("missing 'meta' section".into()));
    }

    let name = req_str(m, "name", "meta")?;
    let version = req_str(m, "version", "meta")?;
    let description = opt_str(m, "description").unwrap_or_default();
    let category = opt_str(m, "category").unwrap_or_default();

    let arch = m["arch"]
        .as_vec()
        .map(|v| {
            v.iter()
                .filter_map(|y| y.as_str().map(ToOwned::to_owned))
                .collect()
        })
        .unwrap_or_default();

    Ok(Meta {
        name,
        version,
        description,
        category,
        icon: opt_str(m, "icon"),
        native_description: opt_str(m, "native_description"),
        license: opt_str(m, "license"),
        homepage: opt_str(m, "homepage"),
        arch,
        broken: m["broken"].as_bool().unwrap_or(false),
        no_cache: m["no_cache"].as_bool().unwrap_or(false),
    })
}

fn parse_deps(doc: &Yaml) -> Result<Deps> {
    let d = &doc["deps"];
    if d.is_badvalue() {
        return Ok(Deps::default());
    }
    Ok(Deps {
        build: parse_dep_list(&d["build"]),
        run: parse_dep_list(&d["run"]),
    })
}

fn parse_dep_list(yaml: &Yaml) -> Vec<String> {
    match yaml.as_vec() {
        None => vec![],
        Some(v) => v
            .iter()
            .filter_map(|y| y.as_str().map(ToOwned::to_owned))
            .collect(),
    }
}

fn parse_src(doc: &Yaml) -> Result<Source> {
    let s = &doc["src"];
    if s.is_badvalue() {
        return Ok(Source::None);
    }

    let fetch_node = &s["fetch"];
    if !fetch_node.is_badvalue() {
        let url = req_str(fetch_node, "url", "src.fetch")?;
        let sha256 = req_str(fetch_node, "sha256", "src.fetch")?;
        let kind = opt_str(fetch_node, "type");
        let mirrors = fetch_node["mirrors"]
            .as_vec()
            .map(|v| {
                v.iter()
                    .filter_map(|y| y.as_str().map(ToOwned::to_owned))
                    .collect()
            })
            .unwrap_or_default();
        return Ok(Source::Fetch(FetchSource {
            url,
            sha256,
            kind,
            mirrors,
        }));
    }

    let git_node = &s["git"];
    if !git_node.is_badvalue() {
        let url = req_str(git_node, "url", "src.git")?;
        let git_ref = opt_str(git_node, "ref").unwrap_or_else(|| "HEAD".into());
        return Ok(Source::Git(GitSource { url, git_ref }));
    }

    Ok(Source::None)
}

fn parse_build(doc: &Yaml) -> Result<Option<Build>> {
    let b = &doc["build"];
    if b.is_badvalue() {
        return Ok(None);
    }

    let system_str = opt_str(b, "system").unwrap_or_else(|| "shell".into());
    let system = system_str
        .parse::<BuildSystem>()
        .map_err(|()| Error::InvalidSchema(format!("unknown build system '{system_str}'")))?;

    let env = parse_env_map(&b["env"])?;

    Ok(Some(Build {
        env,
        system,
        run: opt_str(b, "run"),
        patches: str_list(&b["patches"]),
    }))
}

fn parse_env_map(yaml: &Yaml) -> Result<HashMap<String, String>> {
    let mut map = HashMap::new();
    match yaml {
        Yaml::Hash(h) => {
            for (k, v) in h {
                if let (Some(key), Some(val)) = (k.as_str(), v.as_str()) {
                    map.insert(key.to_owned(), val.to_owned());
                }
            }
        }
        Yaml::Array(a) => {
            for item in a {
                if let Yaml::Hash(h) = item {
                    for (k, v) in h {
                        if let (Some(key), Some(val)) = (k.as_str(), v.as_str()) {
                            map.insert(key.to_owned(), val.to_owned());
                        }
                    }
                }
            }
        }
        Yaml::BadValue | Yaml::Null => {}
        _ => {
            return Err(Error::InvalidSchema(
                "build.env must be a mapping or list of single-key mappings".into(),
            ));
        }
    }
    Ok(map)
}

fn req_str(node: &Yaml, field: &str, section: &str) -> Result<String> {
    node[field]
        .as_str()
        .map(ToOwned::to_owned)
        .ok_or_else(|| Error::InvalidSchema(format!("missing required field '{section}.{field}'")))
}

fn opt_str(node: &Yaml, field: &str) -> Option<String> {
    node[field].as_str().map(ToOwned::to_owned)
}

fn str_list(yaml: &Yaml) -> Vec<String> {
    yaml.as_vec()
        .map(|v| {
            v.iter()
                .filter_map(|y| y.as_str().map(ToOwned::to_owned))
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use crate::schema::FromYaml;

    use super::*;

    const RECIPE: &str = r#"
meta:
  name: "test-pkg"
  version: "1.2.3"
  description: "A test package"
  license: "MIT"

deps:
  build:
    - "make"
  run:
    - "libz"

src:
  fetch:
    url: "https://example.com/test-1.2.3.tar.gz"
    sha256: "deadbeef"

build:
  env:
    CC: clang
  system: make
  make_args:
    - "-j4"
  parallel: true
"#;

    #[test]
    fn parse_full_recipe() {
        let pkg = Package::from_str_content(RECIPE).unwrap();
        assert_eq!(pkg.meta.name, "test-pkg");
        assert_eq!(pkg.meta.version, "1.2.3");
        assert_eq!(pkg.deps.build.len(), 1);
        assert_eq!(pkg.deps.run.len(), 1);

        let Source::Fetch(f) = &pkg.src else {
            panic!("expected fetch source")
        };
        assert_eq!(f.url, "https://example.com/test-1.2.3.tar.gz");

        let b = pkg.build.as_ref().unwrap();
        assert_eq!(b.system, BuildSystem::Shell);
        assert_eq!(b.env.get("CC"), Some(&"clang".to_owned()));
    }
}
