use serde::{Deserialize, Serialize};

use super::package::Package;

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct Repo {
    pub url: Option<String>,
    pub installed_at: Option<i64>,
    pub name: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub homepage: Option<String>,
    pub packages: Option<Vec<Package>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    const KDL: &str = r#"
    name Foo
    description Bar
    icon "https://github.com/ventrica-org.png"
    homepage "https://ventrica.org"
    "#;

    #[test]
    fn parse() -> miette::Result<()> {
        let config: Repo = kdl::de::from_str(KDL)?;
        println!("{:#?}", config);
        Ok(())
    }
}
