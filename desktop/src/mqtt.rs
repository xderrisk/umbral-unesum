use rumqttd::{Broker, Config};
use std::thread;

pub fn init() {
    let config = config::Config::builder()
        .add_source(config::File::from_str(
            include_str!("../rumqttd.toml"),
            config::FileFormat::Toml,
        ))
        .build()
        .unwrap();

    let rumqttd_config: Config = config.try_deserialize().unwrap();

    let mut broker = Broker::new(rumqttd_config);
    thread::spawn(move || {
        broker.start().unwrap();
    });
}
