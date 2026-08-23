mod app;
mod color;
mod config;
mod generation;
mod gpu;
mod simulation;
mod ui;
mod view;

use winit::event_loop::EventLoop;

use crate::app::App;

fn main() {
    env_logger::init();

    let config = config::load();

    let event_loop = EventLoop::new().expect("Failed to initialize the EventLoop");
    let mut app = App::new(&config);
    event_loop
        .run_app(&mut app)
        .expect("Occured an error while running the app");
}
