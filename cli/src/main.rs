use std::str::FromStr;
use std::time::Duration;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;

use emulator::Chip8;

pub fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        println!("usage: chip8 romfile");
        return Ok(())
    }

    let rom = match std::path::PathBuf::from_str(args[1].as_str()) {
        Ok(path) => path,
        Err(_) => {
            println!("romfile does not exist");
            return Ok(())
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

    let mut emu = Chip8::new();
    emu.load(rom)?;

    'running: loop {
        canvas.clear();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                Event::KeyDown { keycode: Some(keycode), .. } => emu.push_key(&keycode),
                _ => {}
            }
        }
        // The rest of the game loop goes here...
        emu.tick(&mut canvas);

        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 1000));
    }

    Ok(())
}
