use esp_idf_sys as sys;
use std::ffi::CString;
use std::thread;
use std::time::Duration;

pub fn print_system_stats() {
    println!("\n=== ESP32 SYSTEM STATUS ===");

    // 1. Check Heap (RAM) Usage
    unsafe {
        let free_heap = sys::esp_get_free_heap_size();
        let min_free_heap = sys::esp_get_minimum_free_heap_size();
        println!("Free Heap: {} bytes | Min Ever Free Heap: {} bytes\n", free_heap, min_free_heap);
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
