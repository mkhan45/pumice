use pumice::error::PumiceResult;
use pumice::winit::{self, DeviceEvent, ElementState, Event, VirtualKeyCode, WindowEvent};
use pumice::GraphicsContext;

extern crate rand;
use rand::prelude::*;

const BIRD_WIDTH: f32 = 0.195;
const BIRD_HEIGHT: f32 = 0.135;

const PIPE_WIDTH: f32 = 0.3;
const PIPE_HEIGHT: f32 = 3.5;
const PIPE_V_GAP: f32 = 0.263;
const PIPE_H_GAP: f32 = 1.3;
const PIPE_SPAWN_RANGE: f32 = 0.55;

const GRAVITY: f32 = 0.0023;
const JUMP_VEL: f32 = -0.075;

const SPEED: f32 = 0.01615;

#[derive(Copy, Clone)]
struct PipePair {
    x: f32,
    midpoint_y: f32,
}

impl PipePair {
    pub fn new(x: f32) -> Self {
        use rand::distributions::Uniform;

        PipePair {
            x,
            midpoint_y: StdRng::from_entropy()
                .sample(Uniform::from(-PIPE_SPAWN_RANGE..PIPE_SPAWN_RANGE)),
        }
    }

    pub fn init() -> [Self; 6] {
        use std::convert::TryInto;
        (0..6)
            .map(|i| PipePair::new(1.0 + i as f32 * PIPE_H_GAP))
            .collect::<Vec<PipePair>>()[..6]
            .try_into()
            .unwrap()
    }
}

struct Data {
    bird_x: f32,
    bird_y: f32,
    bird_vel: f32,
    bird_rot: f32,
    score: usize,
    pipes: [PipePair; 6],
    screen_size: [f32; 2],
}

impl Data {
    pub fn new() -> Self {
        Data {
            bird_x: -0.75,
            bird_y: 0.0,
            bird_vel: -0.02,
            bird_rot: 0.0,
            score: 0,
            pipes: PipePair::init(),
            screen_size: [1024.0, 1200.0],
        }
    }
}

fn update(ctx: &mut GraphicsContext, data: &mut Data) -> PumiceResult<()> {
    if ctx.screen_size_changed {
        let window = ctx.surface.window();
        let screen_size = window.get_inner_size().unwrap();
        ctx.screen_maxes = [(screen_size.width / screen_size.height) as f32, 1.0];
        data.bird_x = -ctx.screen_maxes[0] + BIRD_HEIGHT * 1.1;
    }

    ctx.new_rectangle_rotcenter(
        [data.bird_x - BIRD_WIDTH / 2.0, data.bird_y],
        [BIRD_WIDTH, BIRD_HEIGHT],
        data.bird_rot,
        [1.0, 0.0, 0.0, 1.0],
    )?;

    // update pipes
    {
        data.pipes.iter().for_each(|pipe_pair| {
            let pos1 = [pipe_pair.x, pipe_pair.midpoint_y - PIPE_V_GAP - PIPE_HEIGHT];
            let pos2 = [pipe_pair.x, pipe_pair.midpoint_y + PIPE_V_GAP];

            ctx.new_rectangle(pos1, [PIPE_WIDTH, PIPE_HEIGHT], [0.0, 1.0, 0.0, 1.0])
                .unwrap();
            ctx.new_rectangle(pos2, [PIPE_WIDTH, PIPE_HEIGHT], [0.0, 1.0, 0.0, 1.0])
                .unwrap();
        });

        let max_x = data
            .pipes
            .iter()
            .map(|pipe_pair| pipe_pair.x)
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(ctx.screen_maxes[0]);

        let bird_y = data.bird_y;
        let bird_x = data.bird_x;
        let mut score = data.score;

        data.pipes.iter_mut().for_each(|pipe_pair| {
            pipe_pair.x -= SPEED;

            let rside = bird_x + BIRD_WIDTH / 2.0;
            let rside_diff = rside - pipe_pair.x;

            let lside = bird_x - BIRD_WIDTH / 2.0;
            let lside_diff = lside - (pipe_pair.x);

            if (rside_diff <= PIPE_WIDTH && rside_diff >= 0.0)
                || (lside_diff <= PIPE_WIDTH && lside_diff >= 0.0)
                || bird_y > 1.0
            {
                if bird_y < pipe_pair.midpoint_y - PIPE_V_GAP
                    || bird_y + BIRD_HEIGHT > pipe_pair.midpoint_y + PIPE_V_GAP
                {
                    println!("You Died! Score: {}", score);
                    std::process::exit(0);
                }
            }

            if pipe_pair.x <= -2.0 - PIPE_WIDTH {
                *pipe_pair = PipePair::new(max_x + PIPE_H_GAP);
                score += 1;
            }
        });
        data.score = score;
    }

    data.bird_vel += GRAVITY;
    data.bird_y += data.bird_vel;

    let target_rot = (data.bird_vel * 420.0).min(42.0).max(-42.0);
    data.bird_rot += (target_rot - data.bird_rot) * 0.5;

    Ok(())
}

fn handle_event(winit_event: &winit::Event, data: &mut Data) -> PumiceResult<()> {
    match winit_event {
        Event::DeviceEvent {
            event: DeviceEvent::Key(input),
            ..
        } => {
            let keycode = input.virtual_keycode;
            match keycode {
                Some(VirtualKeyCode::Space) | Some(VirtualKeyCode::Up) => {
                    if input.state == ElementState::Pressed {
                        if data.bird_vel >= 0.00 {
                            data.bird_vel *= 0.5;
                        }

                        data.bird_vel += JUMP_VEL;

                        if data.bird_vel <= -0.00 {
                            data.bird_vel *= 0.5;
                        }
                    }
                }
                _ => {}
            }
        }
        _ => {}
    }
    Ok(())
}

fn main() -> PumiceResult<()> {
    let ctx = GraphicsContext::new();

    let mut data = Data::new();

    ctx.run::<Data>(&mut data, &update, &handle_event, [0.95, 0.95, 0.95, 1.0])
}
