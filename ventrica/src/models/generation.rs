use super::package::Package;
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
