use regex::Regex;

pub fn is_valid(mem: String) -> bool {
    let re = Regex::new(r"^\d+(\.\d+)?[GMKTP]$").unwrap();
    re.is_match(&mem)
}

pub fn can_use(mem: String) -> bool {
    let re = Regex::new(r"(\d+(\.\d+)?)([GMKTP])").unwrap();
    let caps = re.captures(&mem).unwrap();
    let number: f64 = caps[1].parse().unwrap();
    let suffix = &caps[3];

    let multiplier: u64 = match suffix {
        "P" => 1024 * 1024 * 1024 * 1024,
        "T" => 1024 * 1024 * 1024,
        "G" => 1024 * 1024, // Gigabytes to bytes
        "M" => 1024,         // Megabytes to bytes
        "K" => 1,                // Kilobytes to bytes
        _ => unreachable!(),
    };

    // Return the value in bytes (convert from the number with the appropriate multiplier)
    ((number * multiplier as f64) as u64) < sys_info::mem_info().unwrap().free
}
pub fn check_if_valid(limit: String) {
    if !is_valid(limit.clone()) {
        eprintln!("FATAL: {} is an invalid memory limit.", limit);
        std::process::exit(-1);
    }
    if !can_use(limit.clone()) {
        eprintln!("FATAL: memory limit must be lower than the free memory.");
        std::process::exit(-1);
    }
}
