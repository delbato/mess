pub struct Core {
    memory: [u8; 4096],
    registers: [u8; 16]
}

impl Default for Core {
    fn default() -> Self {
        Self {
            memory: [0; 4096],
            registers: [0; 16]
        }
    }
}