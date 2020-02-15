use pumice::winit;
use pumice::GraphicsContext;
use pumice::PumiceResult;

extern crate rand;
use rand::prelude::*;

const GROUND_Y: f32 = 0.6;

const LOW_Y: f32 = 0.5;
const MID_Y: f32 = 0.2;
const HIGH_Y: f32 = -0.1;

const DINO_WIDTH: f32 = 0.125;
const DINO_HEIGHT: f32 = 0.275;

const DINO_DUCK_HEIGHT: f32 = 0.15;
const DINO_DUCK_WIDTH: f32 = 0.2;

enum DinoState {
    Air(f32, f32), // y pos, y velocity
    Ground,
    Duck,
}

#[derive(Copy, Clone)]
enum ObstacleHeight {
    High,
    Mid,
    Low,
}

#[derive(Copy, Clone)]
struct Obstacle {
    x: f32,
    y: f32,
}

impl Obstacle {
    pub fn new(height: ObstacleHeight, x: f32) -> Self {
        let y = match height {
            ObstacleHeight::High => HIGH_Y,
            ObstacleHeight::Mid => MID_Y,
            ObstacleHeight::Low => LOW_Y,
        };

        Obstacle {
            x,
            y
        }
    }

    pub fn init() -> [Self; 12] {
        use std::convert::TryInto;
        (0..12)
            .map(|i| Obstacle::new(rand::random::<ObstacleHeight>(), 1.0 + i as f32 * 0.3))
            .collect::<Vec<Obstacle>>()[..12]
            .try_into()
            .unwrap()
    }
}

struct Data {
    dino: DinoState,
    dino_x: f32,
    score: usize,
    speed: f32,
    obstacles: [Obstacle; 12],
}

fn update(ctx: &mut GraphicsContext, data: &mut Data) -> PumiceResult<()> {
    if ctx.screen_size_changed {
        let window = ctx.surface.window();
        let screen_size = window.get_inner_size().unwrap();
        ctx.screen_maxes = [(screen_size.width / screen_size.height) as f32, 1.0];
    }

    //ground
    ctx.new_rectangle([-ctx.screen_maxes[0], GROUND_Y], [ctx.screen_maxes[0] * 2.0, 1.0], [0.0, 0.0, 0.0, 1.0]);

    //dino
    {
        let (dino_y, dino_width, dino_height) = match data.dino {
            DinoState::Air(y, _) => (y, DINO_WIDTH, DINO_HEIGHT),
            DinoState::Ground => (GROUND_Y - DINO_HEIGHT, DINO_WIDTH, DINO_HEIGHT),
            DinoState::Duck => (GROUND_Y - DINO_DUCK_HEIGHT, DINO_DUCK_WIDTH, DINO_DUCK_HEIGHT),
        };

        ctx.new_rectangle([data.dino_x, dino_y], [dino_width, dino_height], [0.0, 0.0, 0.0, 1.0]);
    }
    ctx.new_rectangle([-ctx.screen_maxes[0], GROUND_Y], [ctx.screen_maxes[0] * 2.0, 1.0], [0.0, 0.0, 0.0, 1.0]);

    data.obstacles.iter().for_each(|obstacle|{
        ctx.new_rectangle(
            [obstacle.x, obstacle.y],
            [0.1, 0.1],
            [0.0, 0.0, 0.0, 1.0],
        );
    });

    let max_x = data.obstacles.iter()
        .map(|obstacle| obstacle.x)
        .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        .unwrap_or(ctx.screen_maxes[0]);

    data.obstacles.iter_mut().for_each(|obstacle|{
        obstacle.x -= 0.025;

        if obstacle.x + 0.1 <= -ctx.screen_maxes[0] {
            obstacle.x = max_x + 0.5;
        }
    });

    Ok(())
}

fn handle_event(winit_event: &winit::Event, data: &mut Data) -> PumiceResult<()> {
    if let winit::Event::DeviceEvent {
        event: winit::DeviceEvent::Key(input),
        ..
    } = winit_event
    {
        let keycode = input.virtual_keycode;
        match keycode {
            Some(winit::VirtualKeyCode::Space) => if input.state == winit::ElementState::Pressed {},
            _ => {}
        }
    }
    Ok(())
}

fn main() -> PumiceResult<()> {
    let ctx = GraphicsContext::new();
    let mut data = Data {
        dino: DinoState::Ground,
        dino_x: -1.5,
        score: 0,
        speed: 0.01,
        obstacles: Obstacle::init(),
    };

    ctx.run::<Data>(&mut data, &update, &handle_event, [0.95, 0.95, 0.95, 1.0])
}
