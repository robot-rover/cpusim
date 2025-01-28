use gdbstub::{arch::Arch, target::{ext::base::{singlethread::SingleThreadBase, BaseOps}, Target, TargetResult}};
use crate::Emulator;

impl Target for Emulator {
    type Arch = gdbstub_arch::arm::Armv4t;

    type Error = &'static str;

    #[inline(always)]
    fn base_ops(&mut self) -> BaseOps<'_, Self::Arch, Self::Error> {
        BaseOps::SingleThread(self)
    }
}

impl SingleThreadBase for Emulator {
    fn read_registers(
        &mut self,
        regs: &mut <Self::Arch as Arch>::Registers,
    ) -> TargetResult<(), Self> {
        todo!()
    }

    fn write_registers(&mut self, regs: &<Self::Arch as Arch>::Registers)
        -> TargetResult<(), Self> {
        todo!()
    }

    fn read_addrs(
        &mut self,
        start_addr: <Self::Arch as Arch>::Usize,
        data: &mut [u8],
    ) -> TargetResult<usize, Self> {
        todo!()
    }

    fn write_addrs(
        &mut self,
        start_addr: <Self::Arch as Arch>::Usize,
        data: &[u8],
    ) -> TargetResult<(), Self> {
        todo!()
    }
}
