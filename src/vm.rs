use super::*;
use crate::util::map::Map;

use std::os::raw::{c_int, c_uint, c_ulong};
use std::os::unix::io::FromRawFd;
use std::io::{ErrorKind, Result};

#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct Region {
    slot: u32,
    flags: MemoryFlags,
    guest_phys_addr: u64,
    memory_size: u64,
    userspace_addr: u64,
}

impl VirtualMachine {
    pub fn new(kvm: &Kvm) -> Result<Self> {
        const KVM_CAP_MULTI_ADDRESS_SPACE: c_int = 118;
        const KVM_GET_VCPU_MMAP_SIZE: c_ulong = 44548;
        const KVM_CHECK_EXTENSION: c_ulong = 44547;
        const KVM_CREATE_VM: c_ulong = 44545;

        let (fd, limit, size) = unsafe {
            let fd = kvm.0.ioctl(KVM_CREATE_VM, 0 as c_ulong)?;
            let fd = fd::Fd::from_raw_fd(fd as c_int);
            let lim = fd.ioctl(KVM_CHECK_EXTENSION, KVM_CAP_MULTI_ADDRESS_SPACE)?;
            let size = kvm.0.ioctl(KVM_GET_VCPU_MMAP_SIZE, ())?;
            (fd, lim, size as usize)
        };

        Ok(Self {
            multi_addr_space: limit,
            vcpu_mmap_size: size,
            mem: HashMap::new(),
            fd
        })
    }

    pub fn add_region<T: 'static + Copy>(
        &mut self,
        space: u16,
        flags: MemoryFlags,
        addr: u64,
        mut map: Map<T>
    ) -> Result<u16> {
        const KVM_SET_USER_MEMORY_REGION: c_ulong = 1075883590;

        if space as c_uint >= self.multi_addr_space {
            return Err(ErrorKind::InvalidInput.into());
        }

        let maps = self.mem.entry(space).or_insert_with(|| Vec::new());
        let slot = maps.len();

        let mut region = Region {
            slot: slot as u32 | ((space as u32) << 16),
            flags: flags.into(),
            guest_phys_addr: addr,
            memory_size: map[..].len() as u64,
            userspace_addr: &mut *map as *mut T as u64,
        };

        unsafe { self.fd.ioctl(KVM_SET_USER_MEMORY_REGION, &mut region)?; }

        maps.push(unsafe { map.cast() });
        Ok(slot as u16)
    }
}
