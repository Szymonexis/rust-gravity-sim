use crate::simulation::{
    particle::Particle,
    particles_generator::particles_generator::{GeneratorConfig, generate_initial_particles},
};

#[derive(Debug)]
pub struct Simulation {
    particles: Vec<Particle>,
}

impl Simulation {
    pub fn init(config: GeneratorConfig) -> Self {
        let particles = generate_initial_particles(config);

        Self { particles }
    }

    pub fn step(self, time_delta: f32) -> Self {


        self
    }

    pub fn get
}
