use std::fs;
use std::io;
use std::path::{Path, PathBuf};

const CONFIG_TEMPLATE: &str = include_str!("../../app-config.json");

const CONFIG_NAME: &str = "app-config.json";
const SCHEMA_NAME: &str = "app-config.schema.json";

const CONFIG_DIR: &str = "gravity-sim-2d";

pub struct ConfigFile {
    pub path: PathBuf,
    pub contents: String,
    pub display: String,
}

pub fn open() -> Option<ConfigFile> {
    let Some(home) = std::env::home_dir() else {
        eprintln!("config: couldn't work out your home directory");
        return None;
    };

    let dir = home.join(".config").join(CONFIG_DIR);

    match seed(&dir, &home) {
        Ok(file) => Some(file),
        Err(err) => {
            eprintln!(
                "config: couldn't set up {}: {err}",
                dir.join(CONFIG_NAME).display()
            );
            None
        }
    }
}

pub fn template() -> &'static str {
    CONFIG_TEMPLATE
}

fn seed(dir: &Path, home: &Path) -> io::Result<ConfigFile> {
    fs::create_dir_all(dir)?;

    let schema = dir.join(SCHEMA_NAME);
    let generated = super::schema();
    if !fs::read_to_string(&schema).is_ok_and(|current| current == generated) {
        fs::write(&schema, &generated)?;
    }

    let path = dir.join(CONFIG_NAME);
    if !fs::exists(&path)? {
        fs::write(&path, CONFIG_TEMPLATE)?;
        println!("config: wrote a fresh settings file to {}", path.display());
    }

    let contents = fs::read_to_string(&path)?;

    Ok(ConfigFile {
        display: shorten(&path, home),
        path,
        contents,
    })
}

fn shorten(path: &Path, home: &Path) -> String {
    match path.strip_prefix(home) {
        Ok(rest) => format!("~/{}", rest.display()),
        Err(_) => path.display().to_string(),
    }
}
