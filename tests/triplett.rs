use flagset::FlagSet;
use ketuvim::*;

const CODE: &[u8] = &[
    0xba, 0xf8, 0x03, // mov $0x3f8, %dx
    0x00, 0xd8,       // add %bl, %al
    0xee,             // out %al, (%dx)
    0xf4,             // hlt
];

#[test]
fn test() {
    let kvm = Kvm::open().unwrap();
    let vm = VirtualMachine::new(&kvm).unwrap();
    let cpu = VirtualCpu::new(&vm).unwrap();

    // Create the code mapping.
    let mut code = util::map::Map::<()>::build(util::map::Access::Shared)
        .protection(util::map::Protection::Read | util::map::Protection::Write)
        .flags(util::map::Flags::Anonymous)
        .extra(0x1000)
        .done().unwrap();

    // Copy in the code.
    code[..CODE.len()].copy_from_slice(CODE);

    // Add the mapping to the VM.
    vm.write().unwrap().add_region(0, FlagSet::default(), 0x1000, code).unwrap();

    // Setup special registers.
    let mut sregs = cpu.read().unwrap().special_registers().unwrap();
    sregs.cs.base = 0;
    sregs.cs.selector = 0;
    cpu.write().unwrap().set_special_registers(sregs).unwrap();

    // Setup registers.
    cpu.write().unwrap().set_registers(arch::Registers {
        rip: 0x1000,
        rax: 2,
        rbx: 2,
        rflags: 0x2,
        ..Default::default()
    }).unwrap();

    let mut output = None;

    loop {
        match cpu.write().unwrap().run().unwrap() {
            Reason::Halt => break,

            Reason::Io(io) => match io {
                ReasonIo::Out { port, data } => match port {
                    0x03f8 => output = Some(data.to_vec()),
                    _ => panic!("Unexpected IO port!"),
                },

                _ => panic!("Unexpected IO!"),
            },
        }
    }

    assert_eq!(output, Some(vec![4]))
}
