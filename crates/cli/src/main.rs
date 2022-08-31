use std::collections::HashMap;
use std::str::FromStr;
use std::time::Duration;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;

use emulator::Chip8;

mod display;

pub fn main() -> std::io::Result<()> {
    let keymap: HashMap<Keycode, emulator::Keycode> = HashMap::from([
        (Keycode::Num7, emulator::Keycode::Num1),
        (Keycode::Num8, emulator::Keycode::Num2),
        (Keycode::Num9, emulator::Keycode::Num3),
        (Keycode::U, emulator::Keycode::Num4),
        (Keycode::I, emulator::Keycode::Num5),
        (Keycode::O, emulator::Keycode::Num6),
        (Keycode::J, emulator::Keycode::Num7),
        (Keycode::K, emulator::Keycode::Num8),
        (Keycode::L, emulator::Keycode::Num9),
        (Keycode::Comma, emulator::Keycode::Num0),
        (Keycode::M, emulator::Keycode::A),
        (Keycode::Period, emulator::Keycode::B),
        (Keycode::Num0, emulator::Keycode::C),
        (Keycode::P, emulator::Keycode::D),
        (Keycode::Semicolon, emulator::Keycode::E),
        (Keycode::Slash, emulator::Keycode::F),
    ]);

    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        println!("usage: chip8 romfile");
        return Ok(());
    }

    let rom = match std::path::PathBuf::from_str(args[1].as_str()) {
        Ok(path) => path,
        Err(_) => {
            println!("romfile does not exist");
            return Ok(());
        }
    };

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("rust-sdl2 demo", 640, 320)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump().unwrap();

    let sdl_render_target = display::SdlRenderTarget::new(Some(canvas));
    let sdl_display = emulator::display::Display::new(sdl_render_target);
    let mut emu = Chip8::new(sdl_display);
    emu.load(rom)?;

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::KeyDown {
                    keycode: Some(keycode),
                    ..
                } => {
                    if let Some(emu_key) = keymap.get(&keycode) {
                        emu.push_key(emu_key)
                    }
                }
                _ => {}
            }
        }
        // The rest of the game loop goes here...
        emu.tick();

        ::std::thread::sleep(Duration::from_millis(16));
    }

    Ok(())
}
