use async_channel;
use rumqttd::{Broker, Config, Notification};
use serde_json::Value;
use std::thread;

pub struct AulaUpdate {
    pub mac: String,
    pub estado: String,
}

// https://rumqtt.bytebeam.io/docs/rumqttd/Guides/Using%20Link%20to%20communicate%20with%20broker/

pub fn init(sender: async_channel::Sender<AulaUpdate>) {
    let config = config::Config::builder()
        .add_source(config::File::from_str(
            include_str!("../Rumqttd.toml"),
            config::FileFormat::Toml,
        ))
        .build()
        .unwrap();

    let rumqttd_config: Config = config.try_deserialize().unwrap();

    let mut broker = Broker::new(rumqttd_config);
    let (mut link_tx, mut link_rx) = broker.link("singlenode").unwrap();
    thread::spawn(move || {
        broker.start().unwrap();
    });

    link_tx.subscribe("unesum/aulas").unwrap();
    thread::spawn(move || {
        loop {
            match link_rx.recv() {
                Ok(Some(notification)) => {
                    if let Notification::Forward(forward) = notification {
                        if let Ok(json) = serde_json::from_slice::<Value>(&forward.publish.payload)
                        {
                            let mac = json["mac"].as_str().unwrap_or("Desconocida").to_string();
                            let estado = json["estado"].as_str().unwrap_or("0").to_string();
                            let update = AulaUpdate { mac, estado };
                            let _ = sender.send_blocking(update);
                        } else {
                            println!("Error: El payload no es un JSON válido.");
                        }
                    }
                }
                Ok(None) => thread::sleep(std::time::Duration::from_millis(10)),
                Err(e) => println!("Error en el receptor: {:?}", e),
            }
        }
    });
}
