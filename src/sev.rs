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

use std::mem::size_of_val;
use std::mem::uninitialized;
use std::os::raw::c_ulong;
use std::os::unix::io::AsRawFd;

use super::*;
use ::sev::{
    firmware::{Error, Indeterminate},
    launch,
};

#[repr(u32)]
#[allow(dead_code)]
#[derive(Copy, Clone)]
enum Code {
    Init = 0,
    EsInit,

    LaunchStart,
    LaunchUpdateData,
    LaunchUpdateVmsa,
    LaunchSecret,
    LaunchMeasure,
    LaunchFinish,

    SendStart,
    SendUpdateData,
    SendUpdateVmsa,
    SendFinish,

    ReceiveStart,
    ReceiveUpdateData,
    ReceiveUpdateVmsa,
    ReceiveFinish,

    GuestStatus,
    DebugDecrypt,
    DebugEncrypt,
    CertExport,
}

type Result<T> = std::result::Result<T, Indeterminate<Error>>;

pub struct Handle(u32);

pub struct Initialized;
pub struct Started(Handle);
pub struct Measured(Handle, launch::Measurement);

pub struct Launch<T> {
    state: T,
    fw: fd::Fd,
    vm: VirtualMachine,
}

impl<T> Launch<T> {
    fn cmd<U>(&self, code: Code, mut data: U) -> Result<U> {
        pub const KVM_MEMORY_ENCRYPT_OP: c_ulong = 3221794490;

        #[repr(C)]
        struct Command {
            code: Code,
            data: u64,
            error: u32,
            fd: u32,
        }

        let mut cmd = Command {
            error: 0,
            data: &mut data as *mut _ as u64,
            fd: self.fw.as_raw_fd() as u32,
            code,
        };

        match unsafe { self.vm.fd.ioctl(KVM_MEMORY_ENCRYPT_OP, &mut cmd) } {
            Ok(_) => Ok(data),
            _ => Err(cmd.error.into()),
        }
    }
}

impl Launch<Initialized> {
    pub fn new(vm: VirtualMachine) -> Result<Self> {
        let fw = fd::Fd::open("/dev/sev")?;
        let l = Launch {
            state: Initialized,
            fw,
            vm,
        };
        l.cmd(Code::Init, ())?;
        Ok(l)
    }

    pub fn start(self, start: launch::Start) -> Result<Launch<Started>> {
        #[repr(C)]
        struct Data {
            handle: u32,
            policy: launch::Policy,
            dh_addr: u64,
            dh_size: u32,
            session_addr: u64,
            session_size: u32,
        }

        let data = Data {
            handle: 0,
            policy: start.policy,
            dh_addr: &start.cert as *const _ as u64,
            dh_size: size_of_val(&start.cert) as u32,
            session_addr: &start.session as *const _ as u64,
            session_size: size_of_val(&start.session) as u32,
        };

        let state = Started(Handle(self.cmd(Code::LaunchStart, data)?.handle));
        Ok(Launch {
            state,
            fw: self.fw,
            vm: self.vm,
        })
    }
}

impl Launch<Started> {
    pub fn update_data(&mut self, data: &[u8]) -> Result<()> {
        #[repr(C)]
        struct Data {
            addr: u64,
            size: u32,
        }

        let data = Data {
            addr: data.as_ptr() as u64,
            size: data.len() as u32,
        };

        self.cmd(Code::LaunchUpdateData, data)?;
        Ok(())
    }

    pub fn measure(self) -> Result<Launch<Measured>> {
        #[repr(C)]
        struct Data {
            addr: u64,
            size: u32,
        }

        let mut measurement: launch::Measurement = unsafe { uninitialized() };
        let data = Data {
            addr: &mut measurement as *mut _ as u64,
            size: size_of_val(&measurement) as u32,
        };

        self.cmd(Code::LaunchMeasure, data)?;

        Ok(Launch {
            state: Measured(self.state.0, measurement),
            fw: self.fw,
            vm: self.vm,
        })
    }
}

impl Launch<Measured> {
    pub fn measurement(&self) -> launch::Measurement {
        self.state.1
    }

    pub fn inject(&self, mut secret: launch::Secret, gaddr: u64, size: u32) -> Result<()> {
        #[repr(C)]
        struct Data {
            headr_addr: u64,
            headr_size: u32,
            guest_addr: u64,
            guest_size: u32,
            trans_addr: u64,
            trans_size: u32,
        }

        let data = Data {
            headr_addr: &mut secret.header as *mut _ as u64,
            headr_size: size_of_val(&secret.header) as u32,
            guest_addr: gaddr,
            guest_size: size,
            trans_addr: secret.ciphertext.as_mut_ptr() as u64,
            trans_size: secret.ciphertext.len() as u32,
        };

        self.cmd(Code::LaunchSecret, data)?;
        Ok(())
    }

    pub fn finish(self) -> Result<(Handle, VirtualMachine)> {
        self.cmd(Code::LaunchFinish, ())?;
        Ok((self.state.0, self.vm))
    }
}
