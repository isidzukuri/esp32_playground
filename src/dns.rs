use esp_idf_svc::mdns::EspMdns;

pub fn initialize_dns(hostname: &str, instance_name: &str) -> EspMdns {
    println!("Initializing DNS...");
    let mut mdns = EspMdns::take().expect("DNS: Failed initilialization of EspMdns");
    mdns.set_hostname(hostname)
        .expect("DNS: Failed to set hostname");
    mdns.set_instance_name(instance_name)
        .expect("DNS: Failed to set instance name");
    mdns.add_service(None, "_http", "_tcp", 80, &[("path", "/")])
        .expect("DNS: Failed to add services");
    println!("DNS responder started: http://{}.local", hostname);
    mdns
}
