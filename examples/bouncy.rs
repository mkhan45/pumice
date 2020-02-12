use pumice::winit;
use pumice::GraphicsContext;
use pumice::PumiceResult;

const RADIUS: f32 = 0.175;

// the struct that holds all the main data for the game
struct Data {
    x: f32,
    y: f32,
    dx: f32,
    dy: f32,
    paused: bool,
}

// the main update function that accepts an &mut GraphicsContext and Data
// drawing and updating data are both done here.
fn update(ctx: &mut GraphicsContext, data: &mut Data) -> PumiceResult<()> {
    {
        let window = ctx.surface.window();
        let win_size = window.get_inner_size().unwrap();
        ctx.screen_maxes = [(win_size.width / win_size.height) as f32, 1.0]
    }
    ctx.new_circle([data.x, data.y], RADIUS, [1.0, 0.0, 0.0, 1.0])?;

    if !data.paused {
        data.x += data.dx;
        data.y += data.dy;

        if data.x + RADIUS >= ctx.screen_maxes[0] || data.x - RADIUS <= -ctx.screen_maxes[0] {
            data.dx *= -1.0;
        }
        if data.y + RADIUS >= 1.0 || data.y - RADIUS <= -1.0 {
            data.dy *= -1.0;
        }
    }
    Ok(())
}

// Right now the winit events aren't preparsed in any way but I might change that
fn handle_event(winit_event: &winit::Event, data: &mut Data) -> PumiceResult<()> {
    if let winit::Event::DeviceEvent {
        event: winit::DeviceEvent::Key(input),
        ..
    } = winit_event
    {
        let keycode = input.virtual_keycode;
        match keycode {
            Some(winit::VirtualKeyCode::Space) => {
                if input.state == winit::ElementState::Pressed {
                    data.paused = !data.paused;
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
        x: 0.0,
        y: 0.0,
        dx: 0.025,
        dy: -0.01,
        paused: false,
    };

    // tell ctx the Data struct to use, the update function, event handling function,
    // and clear color
    ctx.run::<Data>(&mut data, &update, &handle_event, [0.0, 0.0, 0.0, 1.0])
}
