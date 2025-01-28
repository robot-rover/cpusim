mod gdb;

use std::fs;
use std::path::{Path, PathBuf};

use elf::abi::{PF_R, PF_W, PF_X, PT_LOAD};
use elf::endian::AnyEndian;
use elf::ElfBytes;
use unicorn_engine::Unicorn;
use unicorn_engine::unicorn_const::{Arch, Mode, Permission, SECOND_SCALE};

struct Emulator {
    uni: Unicorn<'static, ()>,
}

impl Emulator {
    fn new() -> Self {
        let uni = Unicorn::new(Arch::ARM, Mode::LITTLE_ENDIAN).expect("failed to initialize Unicorn instance");
        Self { uni }
    }

    fn load_elf(&mut self, path: &Path) {
        let file_data = fs::read(path).unwrap();
        let file = ElfBytes::<AnyEndian>::minimal_parse(&file_data).unwrap();

        for prog in file.segments().unwrap().iter() {
            if prog.p_type == PT_LOAD {
                let mut flags = Permission::NONE;
                if prog.p_flags & PF_X != 0 {
                  flags |= Permission::EXEC;
                }
                if prog.p_flags & PF_W != 0 {
                  flags |= Permission::WRITE;
                }
                if prog.p_flags & PF_R != 0 {
                  flags |= Permission::READ;
                }

                let mem_size = usize::try_from(prog.p_memsz).unwrap();

                if prog.p_filesz > 0 {
                    println!("Offset: {:#x}, Size: {:#x}, Flags: {:?}", prog.p_paddr, mem_size, flags);
                    let file_start = usize::try_from(prog.p_offset).unwrap();
                    let file_end = file_start + usize::try_from(prog.p_filesz).unwrap();
                    self.uni.mem_write(prog.p_paddr, &file_data[file_start..file_end]).unwrap();
                }
            }
        }
    }

    fn setup_memory(&mut self) {
        let rx = Permission::READ | Permission::EXEC;
        // ROM
        const ROM_ADDR: u64 = 0x0000_0000;
        const ROM_CONTENTS: &[u8] = &[0u8; 256 * 1024];
        self.uni.mem_map(ROM_ADDR, ROM_CONTENTS.len(), rx).unwrap();
        self.uni.mem_write(ROM_ADDR, ROM_CONTENTS).unwrap();
        // XIP
        const FLASH_SIZE: usize = 8 * 1024 * 1024;
        self.uni.mem_map(0x1000_0000, FLASH_SIZE, rx).unwrap();
        // SRAM
        const RAM_SIZE: usize = 256 * 1024;
        self.uni.mem_map(0x2000_0000, RAM_SIZE, Permission::all()).unwrap();
    }
}

fn main() {
    let path = PathBuf::from("blink_simple.elf");

    let mut emu = Emulator::new();

    emu.setup_memory();
    emu.load_elf(&path);

    //emu.reg_write(RegisterARM::R0, 123).expect("failed write R0");
    //emu.reg_write(RegisterARM::R5, 1337).expect("failed write R5");

    emu.uni.emu_start(0x1000, u64::MAX, 10 * SECOND_SCALE, 0).unwrap();
    println!("PC: {:#x}", emu.uni.pc_read().unwrap());

}
