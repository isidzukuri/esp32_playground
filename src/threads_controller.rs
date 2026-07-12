use std::ffi::CString;
use std::ptr;
use esp_idf_sys::{xTaskCreatePinnedToCore, TaskHandle_t};

pub fn spawn_pinned_task<F>(name: &str, stack_size: u32, core_id: i32, f: F) -> TaskHandle_t
where
    Box<F>: FnOnce() + Send + 'static,
{
    // Trampoline function that FreeRTOS can execute
    extern "C" fn task_trampoline<F>(params: *mut std::ffi::c_void)
    where
        Box<F>: FnOnce() + Send + 'static,
    {
        // Safety: We recreate the Box from the raw pointer passed by FreeRTOS
        let boxed_f = unsafe { Box::from_raw(params as *mut F) };
        boxed_f();
    }

    let c_name = CString::new(name).unwrap();
    let mut task_handle: TaskHandle_t = ptr::null_mut();
    
    // Allocate the closure on the heap and get a raw pointer
    let user_data = Box::into_raw(Box::new(f));

    unsafe {
        xTaskCreatePinnedToCore(
            Some(task_trampoline::<F>),
            c_name.as_ptr(),
            stack_size,
            user_data as *mut std::ffi::c_void,
            5, // Priority (1-25)
            &mut task_handle,
            core_id, // 0 = PRO_CPU, 1 = APP_CPU
        );
    }

    task_handle
}
