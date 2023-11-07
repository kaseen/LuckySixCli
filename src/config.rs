use config::Config;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref CFG: Config = Config::builder()
        .add_source(config::File::with_name("./Config.toml"))
        .build()
        .unwrap();
}

pub fn get_config_key<'a, T: serde::Deserialize<'a>>(key: &str) -> T {
    CFG.get::<T>(key).unwrap()
}
