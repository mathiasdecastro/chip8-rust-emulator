use std::thread::sleep;
use std::time::{Duration, Instant};

use core::Emulator;
use platform::sdl::SdlPlatform;

fn main() {
    let mut emu = Emulator::new();
    let mut platform = SdlPlatform::new(10);
    let mut last_timer = Instant::now();

    emu.init_fontset();
    emu.load_rom("roms/test_opcode.ch8");

    loop {
        emu.keys = platform.get_keys();

        emu.cycle();

        if last_timer.elapsed() >= Duration::from_millis(16) {
            emu.update_timers();
            last_timer = Instant::now();
        }

        if emu.draw_flag {
            platform.draw(emu.display.pixels());
            emu.draw_flag = false;
        }

        sleep(Duration::from_micros(1200));
    }
}
