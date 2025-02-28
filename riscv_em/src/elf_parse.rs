use object::read::elf::FileHeader;
use object::{Endian, Object, ObjectSection, elf};
use std::error::Error;
use std::fs;

pub struct ElfData {
    pub intructions: Vec<u32>,
    pub base_address: u32,
    pub entry_adress: u32,
}

pub fn read_efl(path: &str) -> Result<ElfData, Box<dyn Error>> {
    let file = fs::read(path)?;
    let elf_data = elf::FileHeader32::<object::Endianness>::parse(&*file)?;
    let file_data = object::File::parse(&*file)?;
    let text_data = file_data.section_by_name(".text").unwrap();

    let endianness = elf_data.endian()?;
    let mut assembly = Vec::new();
    let mut read = 0;
    let mut total = 0;
    for i in 0..text_data.size() {
        let byte = text_data.data()?[i as usize] as u32;
        if endianness.is_little_endian() {
            total += byte << (8 * read);
        } else {
            total <<= 8;
            total += byte;
        }
        read += 1;
        if read == 4 {
            assembly.push(total);
            read = 0;
            total = 0;
        }
    }

    Ok(ElfData {
        intructions: assembly,
        base_address: text_data.address() as u32,
        entry_adress: file_data.entry() as u32,
    })
}
