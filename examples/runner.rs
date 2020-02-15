use pumice::winit;
use pumice::GraphicsContext;
use pumice::PumiceResult;

extern crate rand;
use rand::prelude::*;

const GROUND_Y: f32 = 0.6;

const LOW_Y: f32 = 0.5;
const MID_Y: f32 = 0.375;
const HIGH_Y: f32 = 0.275;

const DINO_WIDTH: f32 = 0.125;
const DINO_HEIGHT: f32 = 0.275;

const DINO_DUCK_HEIGHT: f32 = 0.1;
const DINO_DUCK_WIDTH: f32 = 0.2;

const DINO_JUMP_SPEED: f32 = -0.05;
const GRAVITY: f32 = 0.005;
const FLOAT_FRAMES: u8 = 9;

const START_SPEED: f32 = 0.0165;
const SPEED_INC: f32 = 0.0002;

const OBSTACLE_GAP: f32 = 1.0;

#[derive(Copy, Clone, Debug, PartialEq)]
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

impl ObstacleHeight {
    fn new() -> Self {
        use rand::distributions::Uniform;
        let n = StdRng::from_entropy().sample(Uniform::from(0..10u8));
        match n {
            0..=4 => ObstacleHeight::Low,
            5..=7 => ObstacleHeight::Mid,
            8..=10 => ObstacleHeight::High,
            _ => panic!("Wrong rng"),
        }
    }
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

        Obstacle { x, y }
    }

    pub fn init() -> [Self; 12] {
        use std::convert::TryInto;

        (0..12)
            .map(|i| {
                Obstacle::new(
                    ObstacleHeight::new(),
                    1.0 + i as f32 * OBSTACLE_GAP,
                )
            })
            .collect::<Vec<Obstacle>>()[..12]
            .try_into()
            .unwrap()
    }
}

struct Data {
    dino: DinoState,
    dino_x: f32,
    floated_frames: u8,
    score: usize,
    speed: f32,
    obstacles: [Obstacle; 12],
    duck_held: bool,
}

fn update(ctx: &mut GraphicsContext, data: &mut Data) -> PumiceResult<()> {
    if ctx.screen_size_changed {
        let window = ctx.surface.window();
        let screen_size = window.get_inner_size().unwrap();
        ctx.screen_maxes = [(screen_size.width / screen_size.height) as f32, 1.0];

        data.dino_x = -ctx.screen_maxes[0] + DINO_WIDTH;
    }

    data.obstacles.iter().for_each(|obstacle| {
        ctx.new_rectangle([obstacle.x, obstacle.y], [0.1, 0.1], [0.0, 0.0, 0.0, 1.0]);

        let (dino_y, dino_height, dino_width) = match data.dino {
            DinoState::Ground => (GROUND_Y, DINO_HEIGHT, DINO_WIDTH),
            DinoState::Duck => (GROUND_Y, DINO_DUCK_HEIGHT, DINO_DUCK_WIDTH),
            DinoState::Air(y, _) => (y, DINO_HEIGHT, DINO_WIDTH),
        };

        let dino_center_x = data.dino_x + dino_width / 2.0;
        let dino_center_y = dino_y - dino_height / 2.0;

        let obstacle_center_x = obstacle.x + 0.05;
        let obstacle_center_y = obstacle.y + 0.05;

        if dino_center_x - dino_width / 2.0 < obstacle_center_x + 0.05 &&
            dino_center_x + dino_width / 2.0 > obstacle_center_x - 0.05 &&
                dino_center_y - dino_height / 2.0 < obstacle_center_y + 0.05 &&
                dino_center_y + dino_height / 2.0 > obstacle_center_y - 0.05 {
                    println!("You Died! Speed: {}", data.speed);
                    std::process::exit(0);
        }
    });

    //ground
    ctx.new_rectangle(
        [-ctx.screen_maxes[0], GROUND_Y],
        [ctx.screen_maxes[0] * 2.0, 1.0],
        [0.0, 0.0, 0.0, 1.0],
    );

    if let DinoState::Air(y_pos, vel) = data.dino {
        let nvel =
            if y_pos < GROUND_Y && vel > 0.0 && vel < 0.02 && data.floated_frames <= FLOAT_FRAMES {
                data.floated_frames += 1;
                vel + (0.0 - vel) * 0.4
            } else {
                vel
            };

        let gravity = if vel < 0.0 && !data.duck_held {
            GRAVITY / 1.35
        } else if !data.duck_held {
            GRAVITY
        } else {
            GRAVITY * 2.75
        };

        data.dino = DinoState::Air((y_pos + nvel).min(GROUND_Y), nvel + gravity);

        if y_pos >= GROUND_Y && vel >= 0.0 {
            data.dino = DinoState::Ground;
            data.floated_frames = 0;
        }
    }

    if data.duck_held && data.dino == DinoState::Ground {
        data.dino = DinoState::Duck;
    } else if data.dino == DinoState::Duck && !data.duck_held {
        data.dino = DinoState::Ground;
    }

    //dino
    {
        let (dino_y, dino_width, dino_height) = match data.dino {
            DinoState::Air(y, _) => (y - DINO_HEIGHT, DINO_WIDTH, DINO_HEIGHT),
            DinoState::Ground => (GROUND_Y - DINO_HEIGHT, DINO_WIDTH, DINO_HEIGHT),
            DinoState::Duck => (
                GROUND_Y - DINO_DUCK_HEIGHT,
                DINO_DUCK_WIDTH,
                DINO_DUCK_HEIGHT,
            ),
        };

        ctx.new_rectangle(
            [data.dino_x, dino_y],
            [dino_width, dino_height],
            [0.0, 0.0, 0.0, 1.0],
        );
    }
    ctx.new_rectangle(
        [-ctx.screen_maxes[0], GROUND_Y],
        [ctx.screen_maxes[0] * 2.0, 1.0],
        [0.0, 0.0, 0.0, 1.0],
    );


    let max_x = data
        .obstacles
        .iter()
        .map(|obstacle| obstacle.x)
        .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        .unwrap();

    let mut speed = data.speed;
    data.obstacles.iter_mut().for_each(|obstacle| {
        obstacle.x -= speed;

        if obstacle.x + 0.1 <= -ctx.screen_maxes[0] {
            let height = ObstacleHeight::new();
            *obstacle = Obstacle::new(height, max_x + OBSTACLE_GAP);
            speed += SPEED_INC;
        }
    });
    data.speed = speed;

    Ok(())
}

fn handle_event(winit_event: &winit::Event, data: &mut Data) -> PumiceResult<()> {
    if let winit::Event::DeviceEvent {
        event: winit::DeviceEvent::Key(input),
        ..
    } = winit_event
    {
        use winit::VirtualKeyCode;

        let keycode = input.virtual_keycode;
        match keycode {
            Some(VirtualKeyCode::Space) | Some(VirtualKeyCode::Up) | Some(VirtualKeyCode::W) => {
                if input.state == winit::ElementState::Pressed && data.dino == DinoState::Ground
                    || data.dino == DinoState::Duck
                {
                    data.dino = DinoState::Air(GROUND_Y, DINO_JUMP_SPEED);
                }
            }
            Some(VirtualKeyCode::LControl) | Some(VirtualKeyCode::Down) | Some(VirtualKeyCode::S) => {
                if input.state == winit::ElementState::Pressed {
                    data.duck_held = true;
                } else if input.state == winit::ElementState::Released {
                    data.duck_held = false;
                }
            }
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
        floated_frames: 0,
        score: 0,
        speed: START_SPEED,
        obstacles: Obstacle::init(),
        duck_held: false,
    };

    ctx.run::<Data>(&mut data, &update, &handle_event, [0.95, 0.95, 0.95, 1.0])
}
