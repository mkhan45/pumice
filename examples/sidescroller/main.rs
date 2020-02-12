use pumice::winit;
use pumice::GraphicsContext;
use pumice::PumiceResult;

enum DinoState {
    Air(f32),
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
    height: ObstacleHeight,
    x: f32,
}

impl Obstacle {
    pub fn new(height: ObstacleHeight, x: f32) -> Self {
        Obstacle {
            height,
            x,
        }
    }

    pub fn init() -> [Self; 12] {
        use std::convert::TryInto;
        (0..12)
            .map(|i| Obstacle::new(ObstacleHeight::Mid, 1.0))
            .collect::<Vec<Obstacle>>()[..12]
            .try_into()
            .unwrap()
    }
}

struct Data {
    dino: DinoState,
    score: usize,
    speed: f32,
    obstacles: [Obstacle; 12],
}

fn update(ctx: &mut GraphicsContext, data: &mut Data) -> PumiceResult<()> {
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
        score: 0,
        speed: 0.01,
        obstacles: Obstacle::init(),
    };

    ctx.run::<Data>(&mut data, &update, &handle_event, [0.95, 0.95, 0.95, 1.0])
}
