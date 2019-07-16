use std::fs::File;
use std::io::*;
use std::os::raw::{c_uint, c_ulong};
use std::os::unix::io::*;
use std::ptr::null;

pub struct Fd(RawFd);

impl Fd {
    pub fn new(fd: impl IntoRawFd) -> Self {
        Fd(fd.into_raw_fd())
    }

    pub fn open(file: &str) -> Result<Self> {
        Ok(Fd::new(File::open(file)?))
    }

    pub unsafe fn ioctl<T>(
        &self,
        req: impl Into<c_ulong>,
        data: T,
    ) -> Result<c_uint> {
        let r = libc::ioctl(self.as_raw_fd(), req.into(), data, null::<u8>());
        if r < 0 {
            Err(Error::last_os_error())?
        }
        Ok(r as c_uint)
    }
}

impl AsRawFd for Fd {
    fn as_raw_fd(&self) -> RawFd {
        self.0
    }
}

impl FromRawFd for Fd {
    unsafe fn from_raw_fd(fd: RawFd) -> Self {
        Fd(fd)
    }
}

impl IntoRawFd for Fd {
    fn into_raw_fd(self) -> RawFd {
        self.0
    }
}

impl Drop for Fd {
    fn drop(&mut self) {
        if self.0 >= 0 {
            unsafe {
                libc::close(self.0);
            }
        }
    }
}
