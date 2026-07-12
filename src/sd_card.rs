use std::ffi::CString;
use esp_idf_svc::sys::{
    esp_vfs_fat_sdmmc_mount_config_t, esp_vfs_fat_sdspi_mount, sdmmc_card_t, sdmmc_host_t, spi_host_device_t_SPI2_HOST, 
};
use esp_idf_svc::sys::*;
use std::ptr;
pub fn mount_sd_card() {
    unsafe {
        let mut card: *mut sdmmc_card_t = ptr::null_mut();

        println!("SPI bus initialization");

        unsafe {
            let bus_cfg = spi_bus_config_t {
                sclk_io_num: 14, // CLK
                __bindgen_anon_1: spi_bus_config_t__bindgen_ty_1 { mosi_io_num: 15 }, // MOSI
                __bindgen_anon_2: spi_bus_config_t__bindgen_ty_2 { miso_io_num: 2 },  // MISO
                __bindgen_anon_3: spi_bus_config_t__bindgen_ty_3 { quadwp_io_num: -1 },
                __bindgen_anon_4: spi_bus_config_t__bindgen_ty_4 { quadhd_io_num: -1 },
                data4_io_num: -1,
                data5_io_num: -1,
                data6_io_num: -1,
                data7_io_num: -1,
                data_io_default_level: false,
                max_transfer_sz: 4000,
                flags: 0,
                isr_cpu_id: 0,
                intr_flags: 0,
            };

            let ret = spi_bus_initialize(spi_host_device_t_SPI2_HOST, &bus_cfg, 1);
            if ret == ESP_OK {
                println!("SPI bus initialized successfully");
            } else {
                println!("Failed to init SPI bus: {}", ret);
            }
        }


        // SPI device config
        let slot_config = sdspi_device_config_t {
            host_id: spi_host_device_t_SPI2_HOST,
            gpio_cs: 13,
            gpio_cd: -1, // no card detect
            gpio_wp: -1, // no write protect
            gpio_int: -1,
            ..Default::default()
        };

        // Mount config
        let mount_config = esp_vfs_fat_sdmmc_mount_config_t {
            format_if_mount_failed: false, // true, Automatically formats unreadable cards to FAT32
            max_files: 5,
            allocation_unit_size: 16 * 1024,
            disk_status_check_enable: false,
            use_one_fat: false,  
        };

        let mut host = sdmmc_host_t {
            flags: 1 << 3,                                // SDMMC_HOST_FLAG_SPI
            slot: spi_host_device_t_SPI2_HOST as i32,     // SPI2 bus
            max_freq_khz: 20000,
            io_voltage: 3.3,
            driver_strength: 0,
            current_limit: 0,
            init: Some(sdspi_host_init),
            set_bus_width: None,
            get_bus_width: None,
            set_bus_ddr_mode: None,
            set_card_clk: Some(sdspi_host_set_card_clk),
            set_cclk_always_on: None,
            do_transaction: Some(sdspi_host_do_transaction),
            __bindgen_anon_1: Default::default(),
            io_int_enable: None,
            io_int_wait: None,
            command_timeout_ms: 0,
            get_real_freq: Some(sdspi_host_get_real_freq),
            input_delay_phase: 0,
            set_input_delay: None,
            dma_aligned_buffer: ptr::null_mut(),
            pwr_ctrl_handle: ptr::null_mut(),
            get_dma_info: None,
            check_buffer_alignment: Some(sdspi_host_check_buffer_alignment),
            is_slot_set_to_uhs1: None,
        };
        
        let path = CString::new("/sdcard").unwrap();

        let ret = esp_vfs_fat_sdspi_mount(
            path.as_ptr(),
            &host,
            &slot_config,
            &mount_config,
            &mut card,
        );
        
        if ret == ESP_OK {
            println!("SD card mounted at /sdcard");
        } else {
            println!("Failed to mount SD card: {}", ret);
        }
    }
}