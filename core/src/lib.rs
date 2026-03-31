pub struct Emulator {
    pub memory: [u8; 4096]
}

impl Emulator {
    pub fn new() -> Self {
        Self {
            memory: [0; 4096]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_emulator_init() {
        let emu = Emulator::new();
        assert_eq!(emu.memory.len(), 4096);
    }
}