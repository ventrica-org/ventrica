use super::package::Package;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct Profile {
    pub packages: Vec<Package>,
}
