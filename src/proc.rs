use std::{fs, io};
use std::process::Command;

pub fn get_cpuinfo() -> String {
    fs::read_to_string("/proc/cpuinfo").expect("/proc/cpuinfo not found")
}

pub struct Process {
    pub name: String,
    pub cpu_usage: f64,
}

pub fn get_processes_info() -> Vec<Process> {
    let clk_tck = String::from_utf8(Command::new("getconf").arg("CLK_TCK").output().expect("Failed to get CLK_TCK").stdout).unwrap().trim().parse::<f64>().unwrap();
    let uptime = fs::read_to_string("/proc/uptime").expect("/proc/uptime not found").split_once(" ").unwrap().0.parse::<f64>().unwrap();

    let pids: Result<Vec<_>, io::Error> = Ok(
        fs::read_dir("/proc").expect("/proc not found")
        .into_iter()
        .filter(|r| r.is_ok())
        .map(|r| r.unwrap().path())
        .filter(|r| r.is_dir() && r.file_name().unwrap().to_str().unwrap().parse::<i32>().is_ok())
        .map(|r| r.file_name().unwrap().to_str().unwrap().to_string())
        .collect()
    );

    let mut processes: Vec<Process> = Vec::new();

    for pid in pids.unwrap() {
        let a = match fs::read_to_string(format!("/proc/{}/stat", pid)) {
            Ok(f) => f,
            Err(_) => continue,
        };
        // let a = fs::read_to_string(format!("/proc/{}/stat", pid)).unwrap();
        let stat: Vec<&str> = a.split(" ").into_iter().collect();
        let name = stat[1].to_string();
        let starttime = stat[21].to_string().parse::<f64>().unwrap()/clk_tck;
        let ptime: f64 = (stat[13].to_string().parse::<f64>().unwrap() + stat[13].to_string().parse::<f64>().unwrap())/clk_tck;

        processes.push(Process{ name: name, cpu_usage: ptime * 100.0 / (uptime - starttime) });
    }

    processes.sort_by(|a, b| b.cpu_usage.partial_cmp(&a.cpu_usage).unwrap());

    return processes;
}
