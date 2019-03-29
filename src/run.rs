use flagset::{FlagSet, flags};
use crate::arch;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Run {
    // In
    pub request_interrupt_window: bool,
    pub immediate_exit: bool,
    pub padding1: [u8; 6usize],

    // Out
    pub exit_reason: ReasonCode,
    pub ready_for_interrupt_injection: bool,
    pub if_flag: u8,
    pub flags: FlagSet<arch::RunFlags>,

    // In (Pre-KVM-Run), Out (Post-KVM-Run)
    pub cr8: u64,
    pub apic_base: u64,

    #[cfg(target_arch = "s390")]
    pub psw_mask: u64,

    #[cfg(target_arch = "s390")]
    pub psw_addr: u64,

    pub reason: ReasonData,

    pub valid_regs: u64,
    pub dirty_regs: u64,
    pub s: SharedRegisters,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union SharedRegisters {
    pub regs: super::arch::SyncRegisters,
    pub padding: [u8; 2048usize],
}

#[repr(u32)]
#[allow(dead_code)]
#[derive(Debug, Copy, Clone)]
pub enum ReasonCode {
    Unknown = 0,
    Exception = 1,
    Io = 2,
    Hypercall = 3,
    Debug = 4,
    Hlt = 5,
    Mmio = 6,
    IrqWindowOpen = 7,
    Shutdown = 8,
    FailEntry = 9,
    Intr = 10,
    SetTpr = 11,
    TprAccess = 12,
    S390Sieic = 13,
    S390Reset = 14,
    Dcr = 15,
    Nmi = 16,
    InternalError = 17,
    Osi = 18,
    PaprHcall = 19,
    S390Ucontrol = 20,
    Watchdog = 21,
    S390Tsch = 22,
    Epr = 23,
    SystemEvent = 24,
    S390Stsi = 25,
    IoapicEoi = 26,
    HyperV = 27,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union ReasonData {
    pub hw: ReasonUnknown,
    pub fail_entry: ReasonFailEntry,
    pub ex: ReasonException,
    pub io: ReasonIo,
    pub debug: arch::DebugExit,
    pub mmio: ReasonMmio,
    pub hypercall: ReasonHypercall,
    pub tpr_access: ReasonTprAccess,
    pub s390_sieic: ReasonS390Sieic,
    pub s390_reset_flags: FlagSet<ReasonS390ResetFlags>,
    pub s390_ucontrol: ReasonS390Ucontrol,
    pub dcr: ReasonDcr,
    pub internal: ReasonInternalError,
    pub osi: ReasonOsi,
    pub papr_hcall: ReasonPaprHcall,
    pub s390_tsch: ReasonS390Tsch,
    pub epr: ReasonEpr,
    pub system_event: ReasonSystemEvent,
    pub s390_stsi: ReasonS390Stsi,
    pub eoi: ReasonIoApicEoi,
    pub hyperv: HyperVExit,
    pub padding: [u8; 256usize],
    _bindgen_union_align: [u64; 32usize],
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ReasonUnknown {
    pub hardware_exit_reason: u64,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ReasonFailEntry {
    pub hardware_entry_failure_reason: u64,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ReasonException {
    pub exception: u32,
    pub error_code: u32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ReasonIo {
    pub direction: IoDirection,
    pub size: u8,
    pub port: u16,
    pub count: u32,
    pub data_offset: u64,
}

#[repr(u8)]
#[allow(dead_code)]
#[derive(Debug, Copy, Clone)]
pub enum IoDirection {
    In = 0,
    Out = 1,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ReasonMmio {
    pub phys_addr: u64,
    pub data: [u8; 8usize],
    pub len: u32,
    pub is_write: u8,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ReasonHypercall {
    pub nr: u64,
    pub args: [u64; 6usize],
    pub ret: u64,
    pub longmode: u32,
    pub pad: u32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ReasonTprAccess {
    pub rip: u64,
    pub is_write: u32,
    pub pad: u32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ReasonS390Sieic {
    pub icptcode: u8,
    pub ipa: u16,
    pub ipb: u32,
}

flags! {
    pub enum ReasonS390ResetFlags: u64 {
        Por = 1,
        Clear = 2,
        Subsystem = 4,
        CpuInit = 8,
        Ipl = 16,
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ReasonS390Ucontrol {
    pub trans_exc_code: u64,
    pub pgm_code: u32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ReasonDcr {
    pub dcrn: u32,
    pub data: u32,
    pub is_write: u8,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ReasonInternalError {
    pub suberror: ReasonInternalErrorSubError,
    pub ndata: u32,
    pub data: [u64; 16usize],
}

#[repr(u32)]
#[allow(dead_code)]
#[derive(Debug, Copy, Clone)]
pub enum ReasonInternalErrorSubError {
    Emulation = 1,
    SimulEx = 2,
    DeliveryEv = 3,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ReasonOsi {
    pub gprs: [u64; 32usize],
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ReasonPaprHcall {
    pub nr: u64,
    pub ret: u64,
    pub args: [u64; 9usize],
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ReasonS390Tsch {
    pub subchannel_id: u16,
    pub subchannel_nr: u16,
    pub io_int_parm: u32,
    pub io_int_word: u32,
    pub ipb: u32,
    pub dequeued: u8,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ReasonEpr {
    pub epr: u32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ReasonSystemEvent {
    pub kind: ReasonSystemEventKind,
    pub flags: u64,
}

#[repr(u32)]
#[allow(dead_code)]
#[derive(Debug, Copy, Clone)]
pub enum ReasonSystemEventKind {
    Shutdown = 1,
    Reset = 2,
    Crash = 3,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ReasonS390Stsi {
    pub addr: u64,
    pub ar: u8,
    pub reserved: u8,
    pub fc: u8,
    pub sel1: u8,
    pub sel2: u16,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ReasonIoApicEoi {
    pub vector: u8,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct HyperVExit {
    pub kind: HyperVExitKind,
    pub u: HyperVExitUnion,
}

#[repr(u32)]
#[allow(dead_code)]
#[derive(Copy, Clone)]
pub enum HyperVExitKind {
    Synic = 1,
    Hcall = 2,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union HyperVExitUnion {
    pub synic: HyperVExitSynic,
    pub hcall: HyperVExitHcall,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct HyperVExitSynic {
    pub msr: u32,
    pub control: u64,
    pub evt_page: u64,
    pub msg_page: u64,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct HyperVExitHcall {
    pub input: u64,
    pub result: u64,
    pub params: [u64; 2usize],
}
