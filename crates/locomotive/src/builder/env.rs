use std::collections::HashMap;

pub fn build_env(prefix: Option<impl Into<String>>) -> HashMap<String, String> {
    let mut env = HashMap::new();
    let prefix = prefix.map(|p| p.into()).unwrap_or_default();

    env.insert(
        "PATH".into(),
        format!("{prefix}/bin:/usr/bin:/bin:/usr/sbin:/sbin"),
    );

    env.insert("CC".into(), "clang".into());
    env.insert("CXX".into(), "clang++".into());
    env.insert("AR".into(), "ar".into());

    env.insert("CPPFLAGS".into(), format!("-I{prefix}/include"));
    env.insert("CFLAGS".into(), format!("-I{prefix}/include"));
    env.insert("CXXFLAGS".into(), format!("-I{prefix}/include"));

    env.insert("ACLOCAL_PATH".into(), format!("{prefix}/share/aclocal"));
    env.insert(
        "PKG_CONFIG_PATH".into(),
        format!("{prefix}/lib/pkgconfig:{prefix}/share/pkgconfig"),
    );

    env.insert("LDFLAGS".into(), format!("-L{prefix}/lib"));

    #[cfg(target_os = "macos")]
    {
        let deployment_target = if cfg!(target_arch = "x86_64") {
            "10.12"
        } else {
            "11.0"
        };

        env.insert("MACOSX_DEPLOYMENT_TARGET".into(), deployment_target.into());
        env.insert("DYLD_LIBRARY_PATH".into(), format!("{prefix}/lib"));
    }

    env.insert("PREFIX".into(), format!("{prefix}"));

    env
}
