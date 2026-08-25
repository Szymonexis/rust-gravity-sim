#![allow(dead_code, unused_imports, unused_variables)]

use std::{env, fs, path::PathBuf};

mod config {
    pub mod partial {
        use serde::de::DeserializeOwned;
        use serde_json::{Map, Value};

        pub trait FromJsonObject: Default {
            fn from_json_object(obj: &Map<String, Value>, path: &str) -> Self;
        }

        pub fn leaf<T: DeserializeOwned>(
            _obj: &Map<String, Value>,
            _key: &str,
            _path: &str,
            fallback: T,
        ) -> T {
            fallback
        }

        pub fn section<T: FromJsonObject>(_obj: &Map<String, Value>, _key: &str, _path: &str) -> T {
            T::default()
        }

        pub fn warn_unknown_keys(_obj: &Map<String, Value>, _known: &[&str], _path: &str) {}

        pub fn join(_path: &str, _key: &str) -> String {
            String::new()
        }
    }

    pub mod app {
        include!("src/config/app.rs");
    }
    pub mod colors {
        include!("src/config/colors.rs");
    }
    pub mod generation {
        include!("src/config/generation.rs");
    }
    pub mod pan {
        include!("src/config/pan.rs");
    }
    pub mod particles {
        include!("src/config/particles.rs");
    }
    pub mod schema {
        include!("src/config/schema.rs");
    }
    pub mod simulation {
        include!("src/config/simulation.rs");
    }
    pub mod ui {
        include!("src/config/ui.rs");
    }
    pub mod window {
        include!("src/config/window.rs");
    }
    pub mod zoom {
        include!("src/config/zoom.rs");
    }

    pub use app::AppConfig;
    pub use colors::ColorsConfig;
    pub use generation::{
        Area, GenerationConfig, MassStrategy, OverlapStrategy, PositionStrategy, VelocityStrategy,
    };
    pub use pan::PanConfig;
    pub use particles::ParticlesConfig;
    pub use simulation::SimulationConfig;
    pub use ui::UiConfig;
    pub use window::WindowConfig;
    pub use zoom::ZoomConfig;
}

mod math {
    pub mod vec2 {
        include!("src/math/vec2.rs");
    }

    pub use vec2::Vec2;
}

mod simulation {
    pub mod particle {
        include!("src/simulation/particle.rs");
    }

    pub use particle::Particle;
}

fn main() {
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed=src/config");
    println!("cargo::rerun-if-changed=src/math");
    println!("cargo::rerun-if-changed=src/simulation/particle.rs");
    println!("cargo::rerun-if-changed=app-config.schema.json");

    let schema = config::schema::schema();
    let path = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("cargo sets this"))
        .join("app-config.schema.json");

    if fs::read_to_string(&path).is_ok_and(|current| current == schema) {
        return;
    }

    if let Err(err) = fs::write(&path, schema) {
        println!("cargo::warning=couldn't write {}: {err}", path.display());
    }
}
