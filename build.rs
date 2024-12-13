fn main() {
    if cfg!(target_os = "windows") {
        let mut res = winres::WindowsResource::new();
        res.set_icon("src/assets/duniternodemanager.ico");
        res.compile().expect("Failed to compile resources");
    }

    let qt_include_path = std::env::var("DEP_QT_INCLUDE_PATH").unwrap();
    let qt_library_path = std::env::var("DEP_QT_LIBRARY_PATH").unwrap();
    let qt_version = std::env::var("DEP_QT_VERSION")
        .unwrap()
        .parse::<semver::Version>()
        .expect("Parsing Qt version failed");
    if qt_version < semver::Version::new(6, 0, 0) {
        println!("cargo:rustc-cfg=no_qt");
        return;
    }

    let mut config = cpp_build::Config::new();
    for f in std::env::var("DEP_QT_COMPILE_FLAGS")
        .unwrap()
        .split_terminator(';')
    {
        config.flag(f);
    }
    config.include(&qt_include_path);
    if cfg!(target_os = "macos") {
        config.include(format!("{}/QtGui.framework/Headers/", qt_library_path));
    } else {
        config.include(format!("{}/QtGui", qt_include_path));
    }
    config.build("src/main.rs");
}
