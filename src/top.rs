use esp_idf_sys as sys;
use std::collections::HashMap;
use std::ffi::CString;
use std::thread;
use std::time::Duration;
// use std::io::Write;
use std::fmt::Write;

pub fn print_system_stats() {
    println!("\n=== ESP32 SYSTEM STATUS ===");

    // 1. Check Heap (RAM) Usage
    unsafe {
        let free_heap = sys::esp_get_free_heap_size();
        let min_free_heap = sys::esp_get_minimum_free_heap_size();
        println!(
            "Free Heap: {} bytes | Min Ever Free Heap: {} bytes\n",
            free_heap, min_free_heap
        );
    }

    // 2. Allocate buffers for FreeRTOS formatted lists
    // Note: FreeRTOS vTaskList & vTaskGetRunTimeStats require writeable raw char buffers.
    // Ensure the buffers are large enough (~40 bytes per task is a safe guess).
    let mut task_list_buf = vec![0u8; 1024];
    let mut runtime_stats_buf = vec![0u8; 1024];

    unsafe {
        // println!("Task Name\tStatus\tPrio\tStackLft\tID");
        // println!("--------------------------------------------------");
        // sys::vTaskList(task_list_buf.as_mut_ptr() as *mut std::os::raw::c_char);

        // // Find where the C-string actually ends (first null byte)
        // if let Some(end_pos) = task_list_buf.iter().position(|&b| b == 0) {
        //     println!("{}", String::from_utf8_lossy(&task_list_buf[..end_pos]));
        // }

        println!("\nTask Name\tAbs Time (Cycles)\t% CPU");
        println!("--------------------------------------------------");
        sys::vTaskGetRunTimeStats(runtime_stats_buf.as_mut_ptr() as *mut std::os::raw::c_char);

        // Find where the C-string actually ends (first null byte)
        if let Some(end_pos) = runtime_stats_buf.iter().position(|&b| b == 0) {
            println!("{}", String::from_utf8_lossy(&runtime_stats_buf[..end_pos]));
        }
    }
}

pub fn get_system_stats() -> HashMap<String, String> {
    let mut stats = HashMap::new();

    // 1. Gather Heap (RAM) Usage
    unsafe {
        let free_heap = sys::esp_get_free_heap_size();
        let min_free_heap = sys::esp_get_minimum_free_heap_size();

        stats.insert("free_heap".to_string(), free_heap.to_string());
        stats.insert("min_free_heap".to_string(), min_free_heap.to_string());
    }

    // 2. Gather FreeRTOS Task Runtime Stats
    let mut runtime_stats_buf = vec![0u8; 1024];

    unsafe {
        sys::vTaskGetRunTimeStats(runtime_stats_buf.as_mut_ptr() as *mut std::os::raw::c_char);

        // Find where the C-string actually ends (first null byte)
        if let Some(end_pos) = runtime_stats_buf.iter().position(|&b| b == 0) {
            let stats_str = String::from_utf8_lossy(&runtime_stats_buf[..end_pos]);

            // Parse the output table line-by-line
            // Typical line format: "IDLE0          1234567         <1%" or "task_name      54321           12%"
            for line in stats_str.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();

                // Ensure we have at least: Task Name, Absolute Time, and CPU Percentage
                if parts.len() >= 3 {
                    let task_name = parts[0];
                    let abs_time = parts[1];
                    let cpu_pct = parts[2];

                    // Insert task-specific metrics into the HashMap
                    stats.insert(format!("task:{}:cpu", task_name), cpu_pct.to_string());
                    stats.insert(format!("task:{}:time", task_name), abs_time.to_string());
                }
            }
        }
    }

    stats
}

pub fn get_system_stats_as_csv() -> String {
    let stats_map = get_system_stats();
    let mut csv_string = String::with_capacity(512);
    let _ = writeln!(csv_string, "metric,value");
    for (key, value) in stats_map {
        let _ = writeln!(csv_string, "{},{}", key, value);
    }
    csv_string
}
