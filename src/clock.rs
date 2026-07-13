use esp_idf_sys::{timeval, settimeofday};
use std::time::{SystemTime, Duration};

pub fn set_time(start_timestamp_secs: i32){
    let tv = timeval {
        tv_sec: start_timestamp_secs as _,
        tv_usec: 0,
    };
    
    // Safety: settimeofday updates the ESP-IDF standard POSIX clock
    unsafe {
        settimeofday(&tv, std::ptr::null());
    }
    println!("System clock has been manually set!");
}

pub fn get_current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("Clock: Failed to read time")
        .as_secs()
}
