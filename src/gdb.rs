use std::io;
use gdbstub::target::ext::breakpoints::{Breakpoints, BreakpointsOps, SwBreakpointOps, SwBreakpoint};
use std::net::TcpStream;
use gdbstub::{arch::Arch, target::{ext::base::{singlethread::SingleThreadBase, BaseOps}, Target, TargetResult}, conn::{Connection, ConnectionExt}};
use gdbstub::stub::run_blocking;
use unicorn_engine::RegisterARM;
use gdbstub::stub::SingleThreadStopReason;
use crate::Emulator;

impl Target for Emulator {
    type Arch = gdbstub_arch::arm::Armv4t;

    type Error = &'static str;

    #[inline(always)]
    fn base_ops(&mut self) -> BaseOps<'_, Self::Arch, Self::Error> {
        BaseOps::SingleThread(self)
    }

    #[inline(always)]
    fn support_breakpoints(&mut self) -> Option<BreakpointsOps<'_, Self>> {
        Some(self)
    }
}

impl SingleThreadBase for Emulator {
    fn read_registers(
        &mut self,
        regs: &mut <Self::Arch as Arch>::Registers,
    ) -> TargetResult<(), Self> {
        for r_idx in 0usize..=12 {
            regs.r[r_idx] = self.uni.reg_read(RegisterARM::R0 as i32 + r_idx as i32).unwrap().try_into().unwrap();
        }
        regs.sp = self.uni.reg_read(RegisterARM::SP).unwrap().try_into().unwrap();
        regs.lr = self.uni.reg_read(RegisterARM::LR).unwrap().try_into().unwrap();
        regs.pc = self.uni.reg_read(RegisterARM::PC).unwrap().try_into().unwrap();
        regs.cpsr = self.uni.reg_read(RegisterARM::CPSR).unwrap().try_into().unwrap();

        Ok(())
    }

    fn write_registers(&mut self, regs: &<Self::Arch as Arch>::Registers)
        -> TargetResult<(), Self> {
        for r_idx in 0usize..=12 {
             self.uni.reg_write(RegisterARM::R0 as i32 + r_idx as i32, regs.r[r_idx] as u64).unwrap();
        }
        self.uni.reg_write(RegisterARM::SP, regs.sp as u64).unwrap();
        self.uni.reg_write(RegisterARM::LR, regs.lr as u64).unwrap();
        self.uni.reg_write(RegisterARM::PC, regs.pc as u64).unwrap();
        self.uni.reg_write(RegisterARM::CPSR, regs.cpsr as u64).unwrap();

        Ok(())
    }

    fn read_addrs(
        &mut self,
        start_addr: <Self::Arch as Arch>::Usize,
        data: &mut [u8],
    ) -> TargetResult<usize, Self> {
        println!("Reading {:#x} bytes from {:#x}", data.len(), start_addr);
        self.uni.mem_read(start_addr as u64, data).unwrap();
        Ok(data.len())
    }

    fn write_addrs(
        &mut self,
        start_addr: <Self::Arch as Arch>::Usize,
        data: &[u8],
    ) -> TargetResult<(), Self> {
        println!("Writing {:#x} bytes to {:#x}", data.len(), start_addr);
        self.uni.mem_write(start_addr as u64, data).unwrap();
        Ok(())
    }
}

impl Breakpoints for Emulator {
    #[inline(always)]
    fn support_sw_breakpoint(&mut self) -> Option<SwBreakpointOps<'_, Self>> {
        Some(self)
    }
}

impl SwBreakpoint for Emulator {
    fn add_sw_breakpoint(
            &mut self,
            addr: <Self::Arch as Arch>::Usize,
            kind: <Self::Arch as Arch>::BreakpointKind,
        ) -> TargetResult<bool, Self> {
        println!("Adding breakpoint at {:#x}", addr);

        let addr = addr as u64;
        let handle = self.uni.add_code_hook(addr, addr , |uni, addr, instr_size| uni.emu_stop().unwrap()).unwrap();

        assert!(self.breakpoints.insert((addr, kind as u64), handle).is_none());

        Ok(true)
    }

    fn remove_sw_breakpoint(
            &mut self,
            addr: <Self::Arch as Arch>::Usize,
            kind: <Self::Arch as Arch>::BreakpointKind,
        ) -> TargetResult<bool, Self> {
        println!("Removing breakpoint at {:#x}", addr);

        self.breakpoints.remove(&(addr as u64, kind as u64)).unwrap();

        Ok(true)
    }
}

pub(crate) enum EmuEventLoop {}

impl run_blocking::BlockingEventLoop for EmuEventLoop {
    type Target = Emulator;
    type Connection = TcpStream;

    type StopReason = SingleThreadStopReason<u32>;

    fn wait_for_stop_reason(
        target: &mut Self::Target,
        conn: &mut Self::Connection,
    ) -> Result<
        run_blocking::Event<Self::StopReason>,
        run_blocking::WaitForStopReasonError<
            <Self::Target as Target>::Error,
            <Self::Connection as Connection>::Error,
        >,
    > {
        loop {
            let pc = target.uni.reg_read(RegisterARM::PC).unwrap().try_into().unwrap();
            match target.uni.emu_start(pc, u64::MAX, 0, 0) {
                Ok(()) => {
                    if let Some(byte) = conn.peek().unwrap() {
                        return Ok(run_blocking::Event::IncomingData(byte));
                    }
                },
                Err(err) => {
                    return Ok(run_blocking::Event::TargetStopped(SingleThreadStopReason::Exited(err as u8)));
                },
            }
        }
    }

    fn on_interrupt(
        target: &mut Self::Target,
    ) -> Result<Option<Self::StopReason>, <Self::Target as Target>::Error> {
        todo!()
    }

}
