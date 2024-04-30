use std::marker::PhantomData;

use serde::{Deserialize, Serialize};

use super::{MessageExtendable, Returnable};
#[derive(Deserialize, Debug)]
/// expected Response from subscribing to Deribit Exchange
pub struct DeribitResponse {
    pub params: Option<ResponseParams>,
}

#[derive(Deserialize, Debug)]
pub struct ResponseParams {
    pub data: DeribitResponseData,
}

#[derive(Deserialize, Debug)]
pub struct DeribitResponseData {
    pub bids: Vec<(f32, f32)>,
    pub asks: Vec<(f32, f32)>,
    pub instrument_name: String,
}

#[derive(Serialize)]
pub struct DeribitInitMessage {
    method: String,
    params: DeribitInitMessageParams,
}


impl Default for DeribitInitMessage {
    fn default() -> Self {
        Self {
            method: String::from("public/subscribe"),
            params: DeribitInitMessageParams::default(),
        }
    }
}

impl MessageExtendable for DeribitInitMessage {
    fn add_asset(&mut self,asset:&str) {
        self.params.add_asset(asset)
    }
    
     fn to_json(& self) ->  anyhow::Result<String>{
           let json = serde_json::to_string_pretty(&self)?;
           Ok(json)
     }
}
#[derive(Serialize)]
struct DeribitInitMessageParams {
    channels: Vec<String>,
    jsonrpc: String,
    id: i32,
}

impl MessageExtendable for DeribitInitMessageParams {
    fn add_asset(&mut self, asset: &str) {
        if let Some(value) = self.channels.get_mut(0) {
            *value = format!("book.{}.none.20.100ms", { asset });
        }
    }

      fn to_json(& self) ->  anyhow::Result<String>{
           let json = serde_json::to_string_pretty(&self)?;
           Ok(json)
     }
    
}

impl Default for DeribitInitMessageParams {
    fn default() -> Self {
        Self {
            channels: vec!["book.BTC-10MAY24-66000-C.none.20.100ms".to_owned()],
            jsonrpc: String::from("2.0"),
            id: 0,
        }
    }
}

impl DeribitInitMessageParams {
    fn new(asset: &String) -> Self {
        let channel = format!("book.{asset}.none.20.100ms");
        Self {
            channels: vec![channel],
            ..Default::default()
        }
    }
}

/// Implements the `Returnable` trait for the `DeribitResponse` struct.
///
/// The `asks_bids_pair` method returns an `Option` containing the asks and bids
/// from the `DeribitResponseData` struct, if present.
///
/// The `instrument_name` method returns an `Option` containing the instrument
/// name from the `DeribitResponseData` struct, if present.
impl Returnable for DeribitResponse {
    fn asks_bids_pair(&self) -> Option<super::AskBidPairs> {
        if let Some(ResponseParams {
            data:
                DeribitResponseData {
                    bids,
                    asks,
                    instrument_name,
                },
        }) = &self.params
        {
            return Some((asks.to_owned(), bids.to_owned()));
        }
        None
    }

    fn instrument_name(&self) -> Option<String> {
        if let Some(ResponseParams {
            data:
                DeribitResponseData {
                    bids,
                    asks,
                    instrument_name,
                },
        }) = &self.params
        {
            return Some(instrument_name.clone());
        }
        None
    }
}
