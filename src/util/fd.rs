// Copyright 2019 Red Hat
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::os::raw::{c_uint, c_ulong};
use std::os::unix::io::*;
use std::ptr::null;
use std::fs::File;
use std::io::*;

pub struct Fd(RawFd);

impl Fd {
    pub fn new(fd: impl IntoRawFd) -> Self {
        Fd(fd.into_raw_fd())
    }

    pub fn open(file: &str) -> Result<Self> {
        Ok(Fd::new(File::open(file)?))
    }

    pub unsafe fn ioctl<T>(&self, req: impl Into<c_ulong>, data: T) -> Result<c_uint> {
        let r = libc::ioctl(self.as_raw_fd(), req.into(), data, null::<u8>());
        if r < 0 { Err(Error::last_os_error())? }
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
