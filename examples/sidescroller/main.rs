use pumice::winit;
use pumice::GraphicsContext;

enum DinoState {
    Air(f32),
    Ground,
    Duck,
}

enum ObstacleHeight {
    High,
    Mid,
    Low,
}

struct Obstacle {
    height: ObstacleHeight,
    x: f32,
}

struct Data {
    dino: DinoState,
    score: usize,
    speed: f32,
    obstacles: Vec<Obstacle>,
}

fn update(ctx: &mut GraphicsContext, data: &mut Data) {
}

fn handle_event(winit_event: &winit::Event, data: &mut Data) {
    if let winit::Event::DeviceEvent {
        event: winit::DeviceEvent::Key(input),
        ..
    } = winit_event
    {
        let keycode = input.virtual_keycode;
        match keycode {
            Some(winit::VirtualKeyCode::Space) => {
                if input.state == winit::ElementState::Pressed {
                }
            }
            _ => {}
        }
    }
}

fn main() {
    let ctx = GraphicsContext::new();
    let mut data = Data {
    };

    ctx.run::<Data>(&mut data, &update, &handle_event, [0.95, 0.95, 0.95, 1.0]);
}
