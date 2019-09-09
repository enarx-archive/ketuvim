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

use super::*;

use std::io::{Error, ErrorKind, Result};
use std::os::raw::c_ulong;

use crate::util::fd;

impl Kvm {
    pub fn open() -> Result<Self> {
        const KVM_GET_API_VERSION: c_ulong = 44544;

        let fd = fd::Fd::open("/dev/kvm")?;

        match unsafe { fd.ioctl(KVM_GET_API_VERSION, ())? } {
            12 => Ok(Self(fd)),
            v => Err(Error::new(
                ErrorKind::InvalidData,
                format!("invalid kvm version: {}", v),
            ))?,
        }
    }
}
