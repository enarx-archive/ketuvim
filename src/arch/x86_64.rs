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

use bitflags::bitflags;

#[repr(C)]
#[derive(Debug, Copy, Clone, Default)]
pub struct Registers {
    pub rax: u64,
    pub rbx: u64,
    pub rcx: u64,
    pub rdx: u64,
    pub rsi: u64,
    pub rdi: u64,
    pub rsp: u64,
    pub rbp: u64,
    pub r8: u64,
    pub r9: u64,
    pub r10: u64,
    pub r11: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,
    pub rip: u64,
    pub rflags: u64,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Default)]
pub struct Segment {
    pub base: u64,
    pub limit: u32,
    pub selector: u16,
    pub kind: u8,
    pub present: u8,
    pub dpl: u8,
    pub db: u8,
    pub s: u8,
    pub l: u8,
    pub g: u8,
    pub avl: u8,
    pub unusable: u8,
    pub padding: u8,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Default)]
pub struct DescriptorTable {
    pub base: u64,
    pub limit: u16,
    pub padding: [u8; 3],
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Default)]
pub struct SpecialRegisters {
    pub cs: Segment,
    pub ds: Segment,
    pub es: Segment,
    pub fs: Segment,
    pub gs: Segment,
    pub ss: Segment,
    pub tr: Segment,
    pub ldt: Segment,
    pub gdt: DescriptorTable,
    pub idt: DescriptorTable,
    pub cr0: u64,
    pub cr2: u64,
    pub cr3: u64,
    pub cr4: u64,
    pub cr8: u64,
    pub efer: u64,
    pub apic_base: u64,
    pub interrupt_bitmap: [u64; 4],
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct SyncRegisters {
    pub regs: Registers,
    pub sregs: SpecialRegisters,
    pub events: CpuEvents,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct CpuEvents {
    pub exception: CpuException,
    pub interrupt: CpuInterrupt,
    pub nmi: CpuNmi,
    pub sipi_vector: u32,
    pub flags: u32,
    pub smi: CpuSmi,
    pub reserved: [u8; 27usize],
    pub exception_has_payload: u8,
    pub exception_payload: u64,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct CpuException {
    pub injected: u8,
    pub nr: u8,
    pub has_error_code: u8,
    pub pending: u8,
    pub error_code: u32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct CpuInterrupt {
    pub injected: u8,
    pub nr: u8,
    pub soft: u8,
    pub shadow: u8,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct CpuNmi {
    pub injected: u8,
    pub pending: u8,
    pub masked: u8,
    pub pad: u8,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct CpuSmi {
    pub smm: u8,
    pub pending: u8,
    pub smm_inside_nmi: u8,
    pub latched_init: u8,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct DebugExit {
    pub exception: u32,
    pub pad: u32,
    pub pc: u64,
    pub dr6: u64,
    pub dr7: u64,
}

bitflags! {
    pub struct RunFlags: u16 {
        const SMM = 1 << 0;
    }
}
