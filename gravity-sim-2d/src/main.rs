mod app;
mod camera;
mod config;
mod gpu;
mod input;
mod scene;
mod simulation;
mod colors;

use winit::event_loop::EventLoop;

use crate::app::App;

fn main() {
    env_logger::init();

    let app_config = config::load();

    let event_loop = EventLoop::new().expect("Failed to initialize the EventLoop");
    let mut app = App::new(&app_config);
    event_loop
        .run_app(&mut app)
        .expect("Occured an error while running the app");
}
