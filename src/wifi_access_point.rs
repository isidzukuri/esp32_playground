use esp_idf_svc::wifi::{AccessPointConfiguration, AuthMethod, EspWifi};
use esp_idf_svc::nvs::EspNvsPartition;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use heapless::String;

pub fn init_ap(
    modem: esp_idf_hal::modem::Modem,
    sys_loop: EspSystemEventLoop,
    nvs: EspNvsPartition<esp_idf_svc::nvs::NvsDefault>,
) -> EspWifi {
    let mut wifi = EspWifi::new(modem, sys_loop, Some(nvs))
        .expect("Failed to initialize ESP WiFi instance");

    let ap_config = AccessPointConfiguration {
        ssid: "Sensor-Server"
            .try_into()
            .expect("Failed to convert SSID into heapless String"),
        password: "password123"
            .try_into()
            .expect("Failed to convert WiFi password into heapless String"),
        auth_method: AuthMethod::WPA2Personal,
        ..Default::default()
    };

    wifi.set_configuration(&esp_idf_svc::wifi::Configuration::AccessPoint(ap_config))
        .expect("Failed to set WiFi access point configuration");
    wifi.start().expect("Failed to start the WiFi access point");
    
    wifi
}