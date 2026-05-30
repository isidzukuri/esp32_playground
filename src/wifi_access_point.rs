use esp_idf_svc::wifi::{AccessPointConfiguration, AuthMethod, EspWifi, Configuration as WifiConfiguration};
use esp_idf_svc::netif::{EspNetif, NetifConfiguration, NetifStack};
use esp_idf_svc::ipv4::{IpInfo, Subnet};
use esp_idf_svc::nvs::EspNvsPartition;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_hal::peripherals::Peripherals;
use std::net::Ipv4Addr;
use esp_idf_svc::ipv4::RouterConfiguration;

pub fn init_ap(
    modem: esp_idf_hal::modem::Modem,
    sys_loop: EspSystemEventLoop,
    nvs: EspNvsPartition<esp_idf_svc::nvs::NvsDefault>,
) {
    let mut netif_conf = NetifConfiguration::wifi_default_router();

    netif_conf.ip_configuration = Some(esp_idf_svc::ipv4::Configuration::Router(RouterConfiguration{
        dns: Some(Ipv4Addr::new(192, 168, 4, 1)),
        subnet: Subnet{gateway: Ipv4Addr::new(255, 255, 255, 0), mask: esp_idf_svc::ipv4::Mask(24)},
        dhcp_enabled: true,
        secondary_dns: None
    }));

    let ap_netif = EspNetif::new_with_conf(&netif_conf).unwrap();

    // // Create a custom netif with static IP
    // let ap_netif = EspNetif::new_with_conf(&NetifConfiguration {
    //     ip_info: Some(ip_info),
    //     stack: NetifStack::Ap,
    //     ..Default::default()
    // }).unwrap();

    // // Initialize WiFi with the custom AP netif
    // let mut wifi = EspWifi::new(modem, sys_loop, Some(nvs), ap_netif, None).unwrap();
    
    let mut wifi = EspWifi::new(modem, sys_loop, Some(nvs)).unwrap();
    let _ = wifi.swap_netif_ap(ap_netif);

    let ap_config = AccessPointConfiguration {
        ssid: "ESP32-Sensor-AP".try_into().unwrap(),
        password: "password123".try_into().unwrap(),
        auth_method: AuthMethod::WPA2Personal,
        ..Default::default()
    };

    wifi.set_configuration(&WifiConfiguration::AccessPoint(ap_config)).unwrap();
    wifi.start().unwrap();
}
