use crate::instr_parse::InstructionError;

use super::Processor;

#[derive(Debug)]
pub struct SystemCall {
    number: u32,
    args: [i32; 6],
}

impl SystemCall {
    pub fn from(proc: &Processor) -> Self {
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

    pub fn exec(&self, proc: &mut Processor) -> Result<(), InstructionError> {
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
                println!("{s}");
                Ok(())
            }
            93 => Err(InstructionError::End),
            _ => Err(InstructionError::NotSupported),
        }
    }
}
