use chrono::Local;
use escpos::printer::Printer;
use escpos::printer_options::PrinterOptions;
use escpos::utils::*;
use escpos::{driver::*, errors::Result};
use rumqttc::{Client, MqttOptions, QoS};
use std::time::Duration;

#[derive(serde::Deserialize)]
struct BonnetjePayload {
    author: String,
    message: String,
}

fn main() -> Result<()> {
    env_logger::init();

    // get env
    let mqtt_user =
        std::env::var("MQTT_USERNAME").expect("Expected a MQTT username in the environment");
    let mqtt_pass =
        std::env::var("MQTT_PASSWORD").expect("Expected a MQTT password in the environment");
    let mqtt_host = std::env::var("MQTT_HOST").expect("Expected a MQTT host in the environment");
    let mqtt_port = std::env::var("MQTT_PORT").expect("Expected a MQTT port in the environment");

    // setup mqtt
    let mut mqttoptions = MqttOptions::new(
        "bonnetje-daemon",
        mqtt_host,
        mqtt_port.parse::<u16>().unwrap(),
    );
    mqttoptions.set_keep_alive(Duration::from_secs(5));
    mqttoptions.set_credentials(mqtt_user, mqtt_pass);

    let (client, mut connection) = Client::new(mqttoptions, 10);

    // subscribe to bonnetje topic
    client
        .subscribe("bonprinter/bonnetje", QoS::AtMostOnce)
        .unwrap();

    // loop thru notifications
    for (_, notification) in connection.iter().enumerate() {
        match notification.unwrap() {
            rumqttc::Event::Incoming(rumqttc::Packet::Publish(packet)) => {
                log::info!(
                    "Message received on topic {}: {:?}",
                    packet.topic,
                    String::from_utf8(packet.payload.to_vec()).unwrap()
                );

                // parse payload
                let payload: BonnetjePayload = match serde_json::from_slice(&packet.payload) {
                    Ok(p) => p,
                    Err(_) => BonnetjePayload {
                        author: "Unknown".to_string(),
                        message: String::from_utf8(packet.payload.to_vec()).unwrap(),
                    },
                };

                // get printer driver (it disconnects after each print and the library doesn't support reconnecting yet)
                let printerdriver = UsbDriver::open(0x0404, 0x0312, None, None)?;

                let mut printer = Printer::new(
                    printerdriver,
                    Protocol::default(),
                    Some(PrinterOptions::default()),
                );

                let print = match printer.init() {
                    Ok(p) => p,
                    Err(e) => {
                        panic!("Failed to initialize printer: {}", e);
                    }
                };

                // print bonnetje!
                print
                    .smoothing(true)?
                    .justify(JustifyMode::CENTER)?
                    .size(2, 2)?
                    .writeln(&payload.author)?
                    .reset_size()?
                    .feeds(2)?
                    .writeln(&payload.message)?
                    .feeds(2)?
                    .size(1, 1)?
                    .writeln(Local::now().to_string().as_str())?
                    .print_cut()?;
            }
            _ => {}
        }
    }

    Ok(())
}
