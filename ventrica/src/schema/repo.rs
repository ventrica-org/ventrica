use std::path::Path;

use yaml_rust2::YamlLoader;

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::schema::FromYaml;

#[derive(Debug, Clone)]
pub struct RepoConfig {
    pub meta: RepoMeta,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoMeta {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub homepage: Option<String>,
}

impl FromYaml for RepoConfig {
    fn from_file(path: &Path) -> Result<Self> {
        let text = std::fs::read_to_string(path).map_err(|e| Error::path(path, e.to_string()))?;
        Self::from_str_content(&text)
    }

    fn from_str_content(text: &str) -> Result<Self> {
        let docs = YamlLoader::load_from_str(text).map_err(|e| Error::YamlParse(e.to_string()))?;

        let doc = docs
            .into_iter()
            .next()
            .ok_or_else(|| Error::YamlParse("empty repo.yml document".into()))?;

        let m = &doc["meta"];
        if m.is_badvalue() {
            return Err(Error::InvalidSchema(
                "repo.yml missing 'meta' section".into(),
            ));
        }

        let name = m["name"]
            .as_str()
            .ok_or_else(|| Error::InvalidSchema("repo.yml meta.name is required".into()))?
            .to_owned();

        let description = m["description"].as_str().map(ToOwned::to_owned);
        let icon = m["icon"].as_str().map(ToOwned::to_owned);
        let homepage = m["homepage"].as_str().map(ToOwned::to_owned);

        Ok(Self {
            meta: RepoMeta {
                name,
                description,
                icon,
                homepage,
            },
        })
    }
}
