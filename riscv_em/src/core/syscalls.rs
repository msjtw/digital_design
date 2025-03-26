use crate::core::Core;
use crate::core::instr_parse::InstructionError;

#[derive(Debug)]
pub struct SystemCall {
    number: u32,
    args: [i32; 6],
}

impl SystemCall {
    pub fn from(proc: &Core) -> Self {
        SystemCall {
            number: proc.reg_file[17] as u32,
            args: {
                let mut args: [i32; 6] = [0; 6];
                for i in 10..16 {
                    args[i - 10] = proc.reg_file[i];
                }
                args
            },
        }
    }

    pub fn exec(&self, proc: &mut Core) -> Result<(), InstructionError> {
        match self.number {
            64 => {
                if self.args[0] != 1 {
                    return Err(InstructionError::NotSupported);
                }
                let addr = self.args[1];
                let len = self.args[2];
                let mut buff: Vec<u8> = Vec::new();
                for i in addr..(addr + len) {
                    buff.push(proc.memory.get_byte(i as u32));
                }
                let s = match String::from_utf8(buff) {
                    Ok(x) => x,
                    Err(_) => return Err(InstructionError::ExecutionError),
                };
                print!("{s}");
                Ok(())
            }
            93 => Err(InstructionError::End),
            _ => Err(InstructionError::NotSupported),
        }
    }
}
