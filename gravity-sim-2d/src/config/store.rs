//! Where the settings file lives on the user's machine, and how it gets there
//! the first time the app runs.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// The templates the executable carries. The files next to `Cargo.toml` are the
/// source of truth while developing; `include_str!` copies them into the binary
/// so it can seed a machine that has never seen the repository.
const CONFIG_TEMPLATE: &str = include_str!("../../app-config.json");
const SCHEMA_TEMPLATE: &str = include_str!("../../app-config.schema.json");

/// What the two files are called, in the repository and in the config directory
/// alike. Keeping the names identical is what lets the template's `$schema`
/// line survive the copy: it already points at the schema sitting beside it.
const CONFIG_NAME: &str = "app-config.json";
const SCHEMA_NAME: &str = "app-config.schema.json";

/// Where they land, under the home directory. Deliberately the same shape on
/// every platform rather than each OS's own settings location - one path to
/// document, and one place to look when a change isn't taking effect.
const CONFIG_DIR: &str = "gravity-sim-2d";

/// The settings file, as found - or as just created.
pub struct ConfigFile {
    pub path: PathBuf,
    pub contents: String,
    /// `~`-shortened, for the overlay: the panel column is narrow, and most of
    /// the full path is home directory.
    pub display: String,
}

/// Find the settings file, seeding `~/.config/gravity-sim-2d/` from the
/// templates the first time. `None` means the home directory is unknown or
/// unwritable, and the caller should fall back to [`template`].
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

/// The defaults compiled into the binary, for when no file can be reached.
pub fn template() -> &'static str {
    CONFIG_TEMPLATE
}

/// Makes sure `dir` and both files in it exist, and hands back the config's
/// contents. `home` is only there to shorten the path for the overlay.
fn seed(dir: &Path, home: &Path) -> io::Result<ConfigFile> {
    fs::create_dir_all(dir)?;

    // The schema describes what this build understands, so it is the app's file
    // to own and is kept in step with the binary. The config is the opposite:
    // written once, then left alone for the user to edit.
    // Skipping the write when it already matches leaves an untouched file's
    // modified time alone.
    let schema = dir.join(SCHEMA_NAME);
    if !fs::read_to_string(&schema).is_ok_and(|current| current == SCHEMA_TEMPLATE) {
        fs::write(&schema, SCHEMA_TEMPLATE)?;
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
