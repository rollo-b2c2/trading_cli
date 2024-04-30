use super::{MessageExtendable, Returnable};
use anyhow::Ok;
use serde::{Deserialize, Serialize};
#[derive(Serialize)]
pub struct OkexInitMessage {
    op: String,
    args: Vec<OkexInitMessageArg>,
}
impl Default for OkexInitMessage {
    fn default() -> Self {
        Self {
            op: String::from("subscribe"),
            args: vec![OkexInitMessageArg::default()],
        }
    }
}

impl MessageExtendable for OkexInitMessage {
    fn add_asset(&mut self, asset: &str) {
        if let Some(arg) = self.args.get_mut(0) {
            arg.add_asset(asset)
        }
    }

     fn to_json(& self) ->  anyhow::Result<String>{
           let json = serde_json::to_string_pretty(&self)?;
           Ok(json)
     }
}

impl OkexInitMessage {
    pub fn new<T: AsRef<str>>(inst_id: T) -> Self {
        let stri = inst_id.as_ref();
        Self {
            op: String::from("subscribe"),
            args: vec![OkexInitMessageArg::new(stri)],
        }
    }
}

impl Default for OkexInitMessageArg {
    fn default() -> Self {
        Self {
            channel: "books".to_owned(),
            inst_id: "BTC-USD-240427-56000-C".to_owned(),
        }
    }
}

impl MessageExtendable for OkexInitMessageArg {
    fn add_asset(&mut self, asset: &str) {
        self.inst_id = asset.to_owned();
    }
    fn to_json(& self) ->  anyhow::Result<String>{
           let json = serde_json::to_string_pretty(&self)?;
           Ok(json)
     }
}
impl OkexInitMessageArg {
    pub fn new(inst_id: &str) -> Self {
        Self {
            channel: "books".to_owned(),
            inst_id: inst_id.to_owned(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct OkexInitMessageArg {
    pub channel: String,
    pub inst_id: String,
}

#[derive(Deserialize, Debug)]
pub struct OkexResponse {
    pub data: Option<Vec<OkexResponseData>>,
    pub arg: Option<OkexInitMessageArg>,
}

#[derive(Deserialize, Debug)]
pub struct OkexResponseData {
    pub bids: Vec<Vec<String>>,
    pub asks: Vec<Vec<String>>,
}

impl Returnable for OkexResponse {
    fn asks_bids_pair(&self) -> Option<super::AskBidPairs> {
        if let Some(ref data) = &self.data {
            if let Some(OkexResponseData { bids, asks }) = data.first() {
                let bid_floats: Vec<_> = bids
                    .iter()
                    .map(|v| {
                        let first: f32 = v[0].parse().unwrap();
                        let second: f32 = v[1].parse().unwrap();
                        (first, second)
                    })
                    .collect();

                let ask_floats: Vec<_> = asks
                    .iter()
                    .map(|v| {
                        let first: f32 = v[0].parse().unwrap();
                        let second: f32 = v[1].parse().unwrap();
                        (first, second)
                    })
                    .collect();

                return Some((ask_floats, bid_floats));
            }
        }
        None
    }

    fn instrument_name(&self) -> Option<String> {
        if let Some(ref data) = &self.arg {
            return Some(data.inst_id.clone());
        }
        None
    }
}
