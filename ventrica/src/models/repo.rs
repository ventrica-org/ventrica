use super::package::Package;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Repo {
    pub id: Option<i64>,
    pub url: Option<String>,
    pub installed_at: Option<i64>,
    pub name: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub homepage: Option<String>,
    pub packages: Option<Vec<Package>>,
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
            packages: None,
        }
    }
}
