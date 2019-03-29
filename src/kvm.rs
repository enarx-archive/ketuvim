use super::*;

use std::io::{Error, ErrorKind, Result};
use std::os::raw::c_ulong;
use std::fs::File;

use crate::util::fd;

impl Kvm {
    pub fn open() -> Result<Arc<RwLock<Self>>> {
        pub const KVM_GET_API_VERSION: c_ulong = 44544;

        let kvm = Self { fd: fd::Fd::new(File::open("/dev/kvm")?) };

        match unsafe { kvm.fd.ioctl(KVM_GET_API_VERSION, ())? } {
            12 => Ok(Arc::new(RwLock::new(kvm))),
            v => Err(Error::new(
                ErrorKind::InvalidData,
                format!("invalid kvm version: {}", v)
            ))?,
        }
    }
}
