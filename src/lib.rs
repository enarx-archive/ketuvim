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

pub mod arch;
pub mod sev;
pub mod util;

mod cpu;
mod kvm;
mod run;
mod vm;

use std::collections::HashMap;
use std::os::raw::c_uint;

use crate::util::{fd, map};

use bitflags::bitflags;

pub struct Kvm(fd::Fd);

bitflags! {
    #[derive(Default)]
    pub struct MemoryFlags: u32 {
        const LOG_DIRTY_PAGES = 1 << 0;
        const READ_ONLY = 1 << 1;
    }
}

pub struct VirtualMachine {
    fd: fd::Fd,
    vcpu_mmap_size: usize,
    multi_addr_space: c_uint,
    mem: HashMap<u16, Vec<map::Map<()>>>,
}

pub struct VirtualCpu {
    fd: fd::Fd,
    run: map::Map<run::Run>,
}

#[derive(Debug)]
pub enum ReasonIo<'a> {
    In { port: u16, data: &'a mut [u8] },
    Out { port: u16, data: &'a [u8] },
}

#[derive(Debug)]
pub enum Reason<'a> {
    Halt,
    Io(ReasonIo<'a>),
    Mmio {
        addr: u64,
        data: &'a [u8],
        read: bool,
    },
}
