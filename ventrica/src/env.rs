use concat_const::concat;
use const_env::env_item;

#[cfg(target_os = "macos")]
#[env_item]
pub const PREFIX: &'static str = "/opt/ventrica";
#[cfg(target_os = "linux")]
#[env_item]
pub const PREFIX: &'static str = "/ventrica";

pub const VENTRICA_STORE_PATH: &'static str = concat!(PREFIX, "/store");
pub const VENTRICA_REPOS_PATH: &'static str = concat!(PREFIX, "/repos");
pub const VENTRICA_GENERATIONS_PATH: &'static str = concat!(PREFIX, "/generations");
pub const VENTRICA_LIVE_PATH: &'static str = concat!(PREFIX, "/live");
pub const VENTRICA_LIVE_TMP_PATH: &'static str = concat!(PREFIX, "/live.new");
pub const VENTRICA_SOCKET_PATH: &'static str =
    concat!(VENTRICA_LIVE_PATH, "/var/run/ventricad.sock");
