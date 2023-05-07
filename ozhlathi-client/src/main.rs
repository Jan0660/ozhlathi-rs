use clap::Parser;
use sys_info::MemInfo;
#[cfg(windows)]
extern crate winapi;
use ozhlathi_base::{MachineStatus, MemoryStatus};

#[derive(Parser)]
struct CliArgs {
    #[arg(long = "webUrl")]
    web_url: String,
    #[arg(long = "name")]
    name: String,
}

fn main() {
    let args = CliArgs::parse();
    println!("webUrl: {}", args.web_url);
    println!("name: {}", args.name);
    if args.web_url.ends_with('/') {
        panic!("webUrl cannot end with a slash")
    }

    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("Failed to build client");
    let report_url = format!("{}/machine/report", args.web_url);

    loop {
        let memory = sys_info::mem_info().expect("Failed to get memory info");

        let (swap_total, swap_free) = unsafe { get_swap_info(&memory) };
        client
            .post(&report_url)
            .json(&MachineStatus {
                name: args.name.clone(),
                timestamp: None,
                memory: MemoryStatus {
                    total_memory: memory.total * 1024,
                    free_memory: memory.free * 1024,
                    available_memory: if memory.avail != 0 {
                        memory.avail * 1024
                    } else {
                        memory.free * 1024
                    },
                    total_swap: swap_total,
                    free_swap: swap_free,
                },
            })
            .send()
            .expect("Failed to send report");

        std::thread::sleep(std::time::Duration::from_secs(10));
    }
}

#[cfg(windows)]
unsafe fn get_swap_info(_memory: &MemInfo) -> (u64, u64) {
    let mut mem_info: winapi::um::sysinfoapi::MEMORYSTATUSEX = std::mem::zeroed();
    mem_info.dwLength = std::mem::size_of::<winapi::um::sysinfoapi::MEMORYSTATUSEX>() as u32;
    winapi::um::sysinfoapi::GlobalMemoryStatusEx(&mut mem_info);

    (mem_info.ullTotalPageFile, mem_info.ullAvailPageFile)
}

#[cfg(not(windows))]
fn get_swap_info(memory: &MemInfo) -> (u64, u64) {
    (memory.total * 1024, memory.free * 1024)
}
