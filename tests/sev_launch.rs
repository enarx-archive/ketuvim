use ketuvim::{Kvm, VirtualMachine, VirtualCpu, MemoryFlags, Reason, ReasonIo, arch, util::map};
use std::convert::TryFrom;
use codicon::Decoder;

const CODE: &[u8] = &[
    0xba, 0xf8, 0x03, // mov $0x3f8, %dx
    0x00, 0xd8,       // add %bl, %al
    0xee,             // out %al, (%dx)
    0xf4,             // hlt
];

fn fetch_chain(fw: &sev::firmware::Firmware) -> sev::certs::Chain {
    const CEK_SVC: &str = "https://kdsintf.amd.com/cek/id";
    const NAPLES: &str = "https://developer.amd.com/wp-content/resources/ask_ark_naples.cert";

    let mut chain = fw.pdh_cert_export()
        .expect("unable to export SEV certificates");

    let id = fw.get_identifer().expect("error fetching identifier");
    let url = format!("{}/{}", CEK_SVC, id);

    let mut rsp = reqwest::get(&url)
        .expect(&format!("unable to contact server"));
    assert!(rsp.status().is_success());

    chain.cek = sev::certs::sev::Certificate::decode(&mut rsp, ())
        .expect("Invalid CEK!");

    let mut rsp = reqwest::get(NAPLES)
        .expect(&format!("unable to contact server"));
    assert!(rsp.status().is_success());

    sev::certs::Chain {
        ca: sev::certs::ca::Chain::decode(&mut rsp, ())
            .expect("Invalid CA chain!"),
        sev: chain,
    }
}

#[test]
fn test() {
    // Server delivers chain and build to client...
    let fw = sev::firmware::Firmware::open().unwrap();
    let build = fw.platform_status().unwrap().build;
    let chain = fetch_chain(&fw);

    // Client creates session and starts the launch.
    let policy = sev::launch::Policy::default();
    let session = sev::session::Session::try_from(policy).unwrap();
    let start = session.start(chain).unwrap();

    // Server spins up the VM.
    let kvm = Kvm::open().unwrap();
    let mut vm = VirtualMachine::new(&kvm).unwrap();
    let code = map::Map::<()>::build(map::Access::Shared)
        .protection(map::Protection::READ | map::Protection::WRITE)
        .flags(map::Flags::ANONYMOUS)
        .extra(0x1000)
        .done().unwrap();
    let addr = &*code as *const () as u64;
    vm.add_region(0, MemoryFlags::default(), 0x1000, code).unwrap();

    // Server takes a measurement and sends it to the client.
    let launch = ketuvim::sev::Launch::new(vm).unwrap();
    let launch = launch.start(start).unwrap();
    let launch = launch.measure().unwrap();
    let measurement = launch.measurement();

    // Client verifies measurement and delivers secret to server.
    let session = session.measure().unwrap();
    let session = session.verify(build, measurement).unwrap();
    let secret = session.secret(sev::launch::HeaderFlags::default(), CODE).unwrap();

    // Server injects the secret into the VM.
    let len = secret.ciphertext.len() as u32;
    launch.inject(secret, addr, len).unwrap();
    let (_, vm) = launch.finish().unwrap();

    // Setup special registers.
    let mut cpu = VirtualCpu::new(&vm).unwrap();
    let mut sregs = cpu.special_registers().unwrap();
    sregs.cs.base = 0;
    sregs.cs.selector = 0;
    cpu.set_special_registers(sregs).unwrap();

    // Setup registers.
    cpu.set_registers(arch::Registers {
        rip: 0x1000,
        rax: 2,
        rbx: 2,
        rflags: 0x2,
        ..Default::default()
    }).unwrap();

    let mut output = None;

    loop {
        match cpu.run().unwrap() {
            Reason::Halt => break,

            Reason::Io(io) => match io {
                ReasonIo::Out { port, data } => match port {
                    0x03f8 => output = Some(data.to_vec()),
                    _ => panic!("Unexpected IO port!"),
                },

                _ => panic!("Unexpected IO!"),
            },

            r => panic!("Unsupported exit reason: {:?}", r),
        }
    }

    assert_eq!(output, Some(vec![4]))
}
