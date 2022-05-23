use std::collections::HashMap;

use dynasmrt::{
    AssemblyOffset,
    ExecutableBuffer,
};

use crate::codegen::error::{
    Error,
    Result,
};

pub struct Output {
    buffer: ExecutableBuffer,
    function_map: HashMap<String, AssemblyOffset>,
}

impl Output {
    pub fn new(function_map: HashMap<String, AssemblyOffset>, buffer: ExecutableBuffer) -> Self {
        Self {
            buffer,
            function_map,
        }
    }

    pub fn get_ptr(&self, fn_name: &str) -> Result<*const u8> {
        let asm_offset = self.function_map.get(fn_name).ok_or(Error::Unknown)?;
        Ok((&self.buffer[asm_offset.0]) as _)
    }
}
