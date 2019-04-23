use super::*;

use std::io::{Error, ErrorKind, Result};
use std::os::raw::c_ulong;
use std::fs::File;

use crate::util::fd;

impl Kvm {
    pub fn open() -> Result<Self> {
        const KVM_GET_API_VERSION: c_ulong = 44544;

        let fd = fd::Fd::new(File::open("/dev/kvm")?);

        match unsafe { fd.ioctl(KVM_GET_API_VERSION, ())? } {
            12 => Ok(Self(fd)),
            v => Err(Error::new(
                ErrorKind::InvalidData,
                format!("invalid kvm version: {}", v)
            ))?,
        }
    }
}
