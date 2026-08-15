mod drivers;
mod env;
mod extract;
pub mod options;

use std::fs;

pub use env::build_env;
pub use options::{BuildUser, PackageBuilderOptions};

use crate::Error;
use crate::network::NetworkManager;
use drivers::BuildDriver;

pub struct PackageBuilder {
    options: PackageBuilderOptions,
}

impl PackageBuilder {
    pub fn new(options: PackageBuilderOptions) -> Result<Self, Error> {
        // before building, set privileges to the specified user (if any)
        // usually the current home user, full root not supported.
        if let Some(user) = &options.user() {
            user.set_process_new_privileges();
        }

        for dir in [
            &options.build_dir(),
            &options.build_src_dir(),
            &options.build_dest_dir(),
        ] {
            fs::create_dir_all(dir)?;
        }

        Ok(Self { options })
    }

    pub fn build(&self) -> Result<(), Error> {
        let package = self.options.package();

        let dest = if let (Some(sources), Some(sha256)) = (&package.sources, &package.sha256) {
            Some(NetworkManager::new().download_file(
                sources,
                self.options.build_dir(),
                Some(sha256),
            )?)
        } else {
            None
        };

        match package.system.as_deref() {
            Some("application") => {
                if let Some(dest) = dest {
                    drivers::MacOSApplicationDriver.extract(&dest, &self.options)?;
                }
            }
            Some("shell") => {
                if let Some(dest) = dest {
                    drivers::ShellDriver.extract(&dest, &self.options)?;
                }

                drivers::ShellDriver.run(&self.options)?;
            }
            _ => {}
        }

        Ok(())
    }
}

impl Drop for PackageBuilder {
    fn drop(&mut self) {
        if !self.options.keep_build_dir() {
            let _ = fs::remove_dir_all(self.options.build_dir());
        }

        if let Some(user) = &self.options.user() {
            user.set_process_old_privileges();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::Package;

    const RECIPE: &str = r#"
    is_disabled #false
    is_cached #true

    name        openssl
    version     "3.5.5"
    description "Cryptography and SSL/TLS Toolkit"
    license     Apache-2.0
    homepage    "https://www.openssl.org/"
    category    security
    platforms   mac_arm64 mac_x86_64

    sources     "https://github.com/openssl/openssl/releases/download/openssl-3.5.5/openssl-3.5.5.tar.gz"
    sha256      "b28c91532a8b65a1f983b4c28b7488174e4a01008e29ce8e69bd789f28bc2a89"

    system      shell
    build """
        echo $PREFIX
        ./Configure \
            --prefix=$PREFIX \
            darwin64-arm64-cc \
            --openssldir=$PREFIX/etc/ssl \
            no-tests \
            shared
        make
        make install DESTDIR=$DESTDIR
    """
    "#;

    #[test]
    fn test() {
        let package: Package = kdl::de::from_str(RECIPE).unwrap();

        let prefix = "/opt/ventrica/usr";

        let builder_options = PackageBuilderOptions::new(package.clone())
            .set_prefix(prefix)
            .set_keep_build_dir(true)
            .set_env(build_env(Some(prefix)));

        println!("options: {:#?}", builder_options);

        let builder = PackageBuilder::new(builder_options.clone())
            .unwrap()
            .build();

        // TODO: sha of kdl file
        // - <SHA256 of store path name>.kdl // kdl file for store path, containing extra metadata like dep paths
        // - <SHA256 of kdl>                 // store path

        if true {
            let src = builder_options.build_dest_dir().join("opt/ventrica/usr");
            let dest = builder_options.build_dir().join("openssl-3.5.5.var");
            varchive::pack_with_metadata(&src, &dest, Some(&package)).unwrap();
        }

        assert!(builder.is_ok(), "{builder:?}");
    }

    #[test]
    fn contents() {
        pub use std::path::Path;
        let path = Path::new("/tmp/openssl-3.5.5.var");
        let metadata: Package = varchive::read_metadata(&path)
            .expect("metadata missing")
            .expect("package missing");

        println!("metadata: {:#?}", metadata);

        let dest = Path::new("/tmp/openssl-3.5.5-extract");
        varchive::unpack(&path, dest).unwrap();

        let string = kdl::se::to_string(&metadata).unwrap();
        let dest = Path::new("/tmp/openssl-3.5.5-extract.kdl");
        std::fs::write(dest, string).unwrap();
    }
}
