use std::path::{Path, PathBuf};

#[cfg(windows)]
pub fn variable_data_path(name: &str) -> PathBuf {
    // On Windows, we use the AppData directory
    use std::env;
    let app_data = env::var("APPDATA").unwrap_or_else(|_| String::from("."));

    let path = Path::new(&app_data).join(name);

    if !path.exists() {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("Failed to create parent directory");
        }
    }

    path
}

#[cfg(unix)]
// If we are logged in as root, we use /var/lib
// Otherwise, we use $XDG_DATA_HOME or $HOME/.local/share
pub fn variable_data_path(name: &str) -> PathBuf {
    use nix::unistd::Uid;

    if Uid::effective().is_root() {
        PathBuf::from("/var/lib").join(name)
    } else {
        use std::env;
        let path = if env::var("XDG_DATA_HOME").is_ok() {
            PathBuf::from(env::var("XDG_DATA_HOME").unwrap())
        } else {
            PathBuf::from(env::var("HOME").unwrap()).join(".local/share")
        }.join(name);

        if !path.exists() {
            std::fs::create_dir_all(&path).expect("Failed to create directory");
        }

        path
    }
}

pub fn config_path() -> String {
    let mut path = String::new();
    path.push_str("config/");
    path.push_str("adsb/");
    path.push_str("/");
    path
}