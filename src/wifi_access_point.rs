use esp_idf_svc::wifi::{AccessPointConfiguration, AuthMethod, EspWifi};
use esp_idf_svc::nvs::EspNvsPartition;
// use esp_idf_svc::hal::prelude::Peripherals;
use esp_idf_svc::eventloop::EspSystemEventLoop;
// use esp_idf_hal::peripherals::Peripherals;
use heapless::String;

pub fn init_ap(
    modem: esp_idf_hal::modem::Modem,
    sys_loop: EspSystemEventLoop,
    nvs: EspNvsPartition<esp_idf_svc::nvs::NvsDefault>,
) -> EspWifi {
    let mut wifi = EspWifi::new(modem, sys_loop, Some(nvs)).unwrap();

    let ap_config = AccessPointConfiguration {
        ssid: "Sensor-Server".try_into().unwrap(),
        password: "password123".try_into().unwrap(),
        auth_method: AuthMethod::WPA2Personal,
        ..Default::default()
    };

    wifi.set_configuration(&esp_idf_svc::wifi::Configuration::AccessPoint(ap_config)).unwrap();
    wifi.start().unwrap();
    
    wifi
}