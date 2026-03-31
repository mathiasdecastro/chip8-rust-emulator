use core::Emulator;

fn main() {
    let emu = Emulator::new();
    println!("Emulator initialized: {} bytes", emu.memory.len())
}