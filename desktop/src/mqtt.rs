use async_channel;
use gettextrs::gettext;
use rumqttd::{Broker, Config, Notification};
use serde_json::Value;
use std::thread;

pub struct ClassroomUpdate {
    pub mac: String,
    pub status: String,
}

pub fn init(sender: async_channel::Sender<ClassroomUpdate>) {
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

    link_tx.subscribe("unesum/classrooms").unwrap();
    thread::spawn(move || {
        loop {
            match link_rx.recv() {
                Ok(Some(notification)) => {
                    if let Notification::Forward(forward) = notification {
                        if let Ok(json) = serde_json::from_slice::<Value>(&forward.publish.payload)
                        {
                            let mac = json["mac"]
                                .as_str()
                                .unwrap_or(&gettext("Unknown"))
                                .to_string();
                            let status = json["status"].as_str().unwrap_or("0").to_string();
                            let update = ClassroomUpdate { mac, status };
                            let _ = sender.send_blocking(update);
                        } else {
                            println!("{}", gettext("Error: Payload is not a valid JSON."));
                        }
                    }
                }
                Ok(None) => thread::sleep(std::time::Duration::from_millis(10)),
                Err(e) => println!("{}: {:?}", gettext("Receiver error"), e),
            }
        }
    });
}
