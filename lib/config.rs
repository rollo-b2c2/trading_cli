use serde::Deserialize;
///  Config  used to fetch specific assets from different exchanges
/// TODO: for scalability use clap to parse settings with assets-exchange pairs
#[derive(Deserialize)]
pub(crate) struct Settings {
    pub assets: Vec<String>, // our assets, default is BTC/USD
}
