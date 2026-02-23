pub mod db;
pub mod live;
pub mod store;
pub mod var;

pub use store::{
    GENERATIONS_DIR, LIVE_PREFIX, REPOS_DIR, STORE_DIR, STORE_ROOT, atomic_move, copy_dir_all,
    link_tree, seal, sha256_file, simple_store_name, simple_store_path, unseal, verify_sha256,
};

pub trait Sealable {
    fn seal(&self) -> crate::error::Result<()>;
    fn unseal(&self) -> crate::error::Result<()>;
}

impl Sealable for std::path::Path {
    fn seal(&self) -> crate::error::Result<()> {
        store::seal(self)
    }

    fn unseal(&self) -> crate::error::Result<()> {
        store::unseal(self)
    }
}
