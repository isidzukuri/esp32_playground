use esp_idf_svc::wifi::{AccessPointConfiguration, AuthMethod, EspWifi};
use esp_idf_svc::nvs::{EspNvsPartition, NvsDefault};
use esp_idf_svc::eventloop::EspSystemEventLoop;
use heapless::String;

pub fn init_ap<'a>(
    modem: esp_idf_hal::modem::Modem<'a>,
    ssid: &'a str,
    password: &'a str
) -> EspWifi<'a> {
    let sys_loop = EspSystemEventLoop::take().unwrap();
    let nvs = EspNvsPartition::<NvsDefault>::take().unwrap();
    let mut wifi = EspWifi::new(modem, sys_loop, Some(nvs))
        .expect("Failed to initialize ESP WiFi instance");

    let ap_config = AccessPointConfiguration {
        ssid: ssid
            .try_into()
            .expect("Failed to convert SSID into heapless String"),
        password: password
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