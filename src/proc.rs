use std::{process::{Command, Stdio}, collections::VecDeque};

pub fn get_cpuinfo(last_idle: &mut Vec<f64>, last_total: &mut Vec<f64>) -> Vec<f64> {
    let cat_proc_stat = Command::new("cat")
        .arg("/proc/stat")
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let grep = Command::new("grep")
        .arg("cpu")
        .stdin(Stdio::from(cat_proc_stat.stdout.unwrap()))
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let data = String::from_utf8(grep.wait_with_output().unwrap().stdout).unwrap();

    let mut v: Vec<&str> = data.split("\n").collect();
    let mut usage_data = vec![];
    v.pop();

    if v.len() > last_idle.len() {
        last_idle.resize(v.len(), 0.0);
        last_total.resize(v.len(), 0.0);
    }

    for i in 1..v.len() {
        let mut d: VecDeque<&str> = v[i].split(" ").collect();
        d.pop_front();
        let mut sum = 0.0;
        for i in &d {
            if let Some(f) = i.to_string().parse::<f64>().ok() {
                sum += f;
            }
        }

        let idle = d[3].to_string().parse::<f64>().unwrap();
        let idle_delta = idle - last_idle[i];
        let total_delta = sum - last_total[i];
        let usage = (1.0 - (idle_delta / total_delta)) * 100.0;
        last_total[i] = sum;
        last_idle[i] = idle;
        usage_data.push(usage);
    }

    let mut res = String::from("");
    let mut vec = vec![];
    for u in 0..4 {
        vec.push(usage_data[u]);
        let new = format!("CPU{}: {}\n", u, usage_data[u]);
        res.push_str(new.as_str());
    }

    vec
}

pub fn get_meminfo() -> String {
    String::from_utf8(Command::new("free").output().expect("Failed to run free -h").stdout).unwrap()
}

pub fn get_uname() -> String {
    String::from_utf8(Command::new("uname").arg("-a").output().expect("Failed to run uname -a").stdout).unwrap()
}

pub fn get_monitor_info() -> String {
    // ps --sort -pcpu -e -o pid,uname,pcpu,pmem,time,comm
    String::from_utf8(Command::new("ps").args(["--sort", "-pcpu", "-e", "-o", "pid,uname,pcpu,pmem,time,comm"]).output().expect("Failed to run ps -sort -pcpu -e -o pid,ppid,uname,pcpu,pmem,time,comm").stdout).unwrap()
}

pub fn open_terminal() {
    Command::new("x-terminal-emulator").spawn().unwrap();
}
