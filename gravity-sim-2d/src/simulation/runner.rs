use std::mem;
use std::sync::atomic::{AtomicBool, AtomicI32, AtomicU32, Ordering};
use std::sync::{Arc, Mutex, PoisonError};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use crate::config::{SimulationConfig, SimulationMethod};
use crate::simulation::{Particle, Speed, World};

const SPIN_MARGIN: Duration = Duration::from_micros(500);
const MAX_SLEEP: Duration = Duration::from_millis(50);
const IDLE_POLL: Duration = Duration::from_millis(2);

struct Slot {
    particles: Vec<Particle>,
    tick: i64,
    fresh: bool,
}

struct Shared {
    slot: Mutex<Slot>,
    running: AtomicBool,
    paused: AtomicBool,
    speed: AtomicU32,
    steps: AtomicI32,
}

pub struct SimulationHandle {
    thread: Option<JoinHandle<()>>,
    shared: Arc<Shared>,
    local: Vec<Particle>,
    tick: i64,
    speed: Speed,
}

impl SimulationHandle {
    pub fn spawn(config: SimulationConfig, world: World) -> Self {
        let count = world.particles().len();
        let speed = Speed::NORMAL;

        let shared = Arc::new(Shared {
            slot: Mutex::new(Slot {
                particles: world.particles().to_vec(),
                tick: 0,
                fresh: true,
            }),
            running: AtomicBool::new(true),
            paused: AtomicBool::new(false),
            speed: AtomicU32::new(speed.value().to_bits()),
            steps: AtomicI32::new(0),
        });

        let thread = thread::Builder::new()
            .name("simulation".to_owned())
            .spawn({
                let shared = Arc::clone(&shared);
                move || run(world, config, &shared)
            })
            .expect("Failed to spawn the simulation thread");

        Self {
            thread: Some(thread),
            shared,
            local: Vec::with_capacity(count),
            tick: 0,
            speed,
        }
    }

    pub fn try_recv(&mut self) -> Option<&[Particle]> {
        {
            let mut slot = self
                .shared
                .slot
                .lock()
                .unwrap_or_else(PoisonError::into_inner);
            if !slot.fresh {
                return None;
            }

            mem::swap(&mut slot.particles, &mut self.local);
            slot.fresh = false;
            self.tick = slot.tick;
        }

        Some(&self.local)
    }

    pub fn tick(&self) -> i64 {
        self.tick
    }

    pub fn is_paused(&self) -> bool {
        self.shared.paused.load(Ordering::Relaxed)
    }

    pub fn toggle_pause(&self) {
        let paused = self.shared.paused.load(Ordering::Relaxed);
        self.shared.paused.store(!paused, Ordering::Relaxed);
    }

    pub fn speed(&self) -> Speed {
        self.speed
    }

    pub fn faster(&mut self) {
        self.speed.faster();
        self.publish_speed();
    }

    pub fn slower(&mut self) {
        self.speed.slower();
        self.publish_speed();
    }

    pub fn step(&self, ticks: i32) {
        self.shared.steps.fetch_add(ticks, Ordering::Relaxed);
    }

    fn publish_speed(&self) {
        self.shared
            .speed
            .store(self.speed.value().to_bits(), Ordering::Relaxed);
    }
}

impl Drop for SimulationHandle {
    fn drop(&mut self) {
        self.shared.running.store(false, Ordering::Relaxed);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

fn run(mut world: World, config: SimulationConfig, shared: &Shared) {
    let base_period = config.tick_period();
    let delta = base_period.as_secs_f32();
    let max_catch_up = config.max_catch_up_ticks.max(1);
    let method = config.method;

    let mut scratch = Vec::with_capacity(world.particles().len());
    let mut next_tick = Instant::now();

    while shared.running.load(Ordering::Relaxed) {
        if shared.paused.load(Ordering::Relaxed) {
            step_manually(&mut world, shared, &mut scratch, delta, method);
            next_tick = Instant::now() + base_period;
            thread::sleep(IDLE_POLL);
            continue;
        }

        let speed = f32::from_bits(shared.speed.load(Ordering::Relaxed));
        if speed == 0.0 {
            next_tick = Instant::now() + base_period;
            thread::sleep(IDLE_POLL);
            continue;
        }

        let period = base_period.div_f32(speed.abs());
        let signed_delta = delta * speed.signum();

        let mut ticks = 0;
        while ticks < max_catch_up && Instant::now() >= next_tick {
            world.on_tick(signed_delta, method);
            next_tick += period;
            ticks += 1;
        }

        if ticks > 0 {
            publish(shared, &mut scratch, &world);
        }

        if Instant::now() >= next_tick {
            next_tick = Instant::now() + period;
        }

        sleep_until(next_tick, shared);
    }
}

fn step_manually(
    world: &mut World,
    shared: &Shared,
    scratch: &mut Vec<Particle>,
    delta: f32,
    method: SimulationMethod,
) {
    let steps = shared.steps.swap(0, Ordering::Relaxed);
    if steps == 0 {
        return;
    }

    let direction = steps.signum() as f32;
    for _ in 0..steps.unsigned_abs() {
        world.on_tick(delta * direction, method);
    }

    publish(shared, scratch, world);
}

fn publish(shared: &Shared, scratch: &mut Vec<Particle>, world: &World) {
    scratch.clear();
    scratch.extend_from_slice(world.particles());

    let mut slot = shared.slot.lock().unwrap_or_else(PoisonError::into_inner);
    mem::swap(&mut slot.particles, scratch);
    slot.tick = world.tick_count();
    slot.fresh = true;
}

fn sleep_until(deadline: Instant, shared: &Shared) {
    loop {
        let Some(remaining) = deadline.checked_duration_since(Instant::now()) else {
            return;
        };

        match remaining.checked_sub(SPIN_MARGIN) {
            Some(sleepable) => thread::sleep(sleepable.min(MAX_SLEEP)),
            None => std::hint::spin_loop(),
        }

        if !shared.running.load(Ordering::Relaxed) || shared.paused.load(Ordering::Relaxed) {
            return;
        }
    }
}
