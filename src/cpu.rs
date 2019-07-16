use super::*;

use crate::util::fd::Fd;
use crate::{arch, run};

use std::io::Result;
use std::mem::{size_of, size_of_val};
use std::os::raw::{c_int, c_ulong};
use std::os::unix::io::FromRawFd;

impl VirtualCpu {
    pub fn new(vm: &VirtualMachine) -> Result<Self> {
        const KVM_CREATE_VCPU: c_ulong = 44609;

        let fd = unsafe { vm.fd.ioctl(KVM_CREATE_VCPU, 0 as c_ulong)? };
        let fd = unsafe { Fd::from_raw_fd(fd as c_int) };

        let run = map::Map::build(map::Access::Shared)
            .protection(map::Protection::READ | map::Protection::WRITE)
            .extra(vm.vcpu_mmap_size - size_of::<run::Run>())
            .file(&fd, 0)
            .done()?;

        Ok(Self { fd, run })
    }

    pub fn registers(&self) -> Result<arch::Registers> {
        const KVM_GET_REGS: c_ulong = 2156965505;

        let mut regs = arch::Registers::default();
        unsafe {
            self.fd.ioctl(KVM_GET_REGS, &mut regs)?;
        }
        Ok(regs)
    }

    pub fn set_registers(&mut self, regs: arch::Registers) -> Result<()> {
        const KVM_SET_REGS: c_ulong = 1083223682;

        unsafe {
            self.fd.ioctl(KVM_SET_REGS, &regs)?;
        }
        Ok(())
    }

    pub fn special_registers(&self) -> Result<arch::SpecialRegisters> {
        const KVM_GET_SREGS: c_ulong = 2167975555;

        let mut regs = arch::SpecialRegisters::default();
        unsafe {
            self.fd.ioctl(KVM_GET_SREGS, &mut regs)?;
        }
        Ok(regs)
    }

    pub fn set_special_registers(
        &mut self,
        regs: arch::SpecialRegisters,
    ) -> Result<()> {
        const KVM_SET_SREGS: c_ulong = 1094233732;

        unsafe {
            self.fd.ioctl(KVM_SET_SREGS, &regs)?;
        }
        Ok(())
    }

    pub fn run<'b>(&'b mut self) -> Result<Reason<'b>> {
        const KVM_RUN: c_ulong = 44672;

        unsafe {
            self.fd.ioctl(KVM_RUN, 0)?;
        }

        Ok(match (*self.run).exit_reason {
            run::ReasonCode::Hlt => Reason::Halt,

            run::ReasonCode::Io => {
                let io = unsafe { &(*self.run).reason.io };

                let port = io.port;
                let size = io.size as usize;
                let start = io.data_offset as usize;
                let count = io.count as usize;

                let start = start - size_of::<run::Run>();

                #[allow(unreachable_patterns)]
                match io.direction {
                    run::IoDirection::In => {
                        let data = &mut self.run[start..][..size * count];
                        Reason::Io(ReasonIo::In { port, data })
                    }

                    run::IoDirection::Out => {
                        let data = &self.run[start..][..size * count];
                        Reason::Io(ReasonIo::Out { port, data })
                    }

                    d => panic!("Unsupported direction: {:?}", d),
                }
            }

            run::ReasonCode::Mmio => {
                let mmio = unsafe { &(*self.run).reason.mmio };

                assert!(mmio.len < size_of_val(&mmio.data) as u32);

                Reason::Mmio {
                    addr: mmio.phys_addr,
                    data: &mmio.data[..mmio.len as usize],
                    read: mmio.is_write == 0,
                }
            }

            r => panic!("Unsupported exit reason: {:?}", r),
        })
    }
}
