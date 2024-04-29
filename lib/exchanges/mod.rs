mod okex;
pub use okex::*;
mod deribit;
pub use deribit::*;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio_tungstenite::tungstenite::http::response;
use futures_util::lock::Mutex;
#[derive(Serialize)]
/// exchange settings to use for connnections
pub enum ExchangePairs<'a> {
    // (name,url, initial_message)
    Deribit(&'a str, &'a str,DeribitInitMessage),
    Okex(&'a str, &'a str,OkexInitMessage),
}

impl<'a> ExchangePairs<'a> {
    pub fn init_message(&self) -> Result<String, Box<dyn std::error::Error>> {
        match &self {
            ExchangePairs::Deribit(_, _,response) => Ok(serde_json::to_string_pretty(response)?),
            ExchangePairs::Okex(_,_,response ) => Ok(serde_json::to_string_pretty(response)?)
        }
    }

    pub fn response (&mut self) -> &mut dyn MessageExtendable {
        match self {
            ExchangePairs::Deribit(_,_, response) => response,
            ExchangePairs::Okex(_,_,response ) => response
        }
    }
    pub fn get_url(&self) -> &'a str {
         match &self {
            ExchangePairs::Deribit(_,url, _) => url,
            ExchangePairs::Okex(_,url,_ ) => url 
        }
    }

     pub fn get_name(&self) -> &'a str {
         match &self {
            ExchangePairs::Deribit(name,_, _) => name,
            ExchangePairs::Okex(name,_,_ ) => name 
        }
     }
}

/// ask-bid pairs in the form of a tuple of (asks, bids)
type AskBidPairs = (Vec<(f32, f32)>, Vec<(f32, f32)>);

/// make return values easier to use, we only care about the bids, asks and instrument name fields
/// This gives us one unified interface for each exchange's response to use
pub trait Returnable {
    fn asks_bids_pair(&self) -> Option<AskBidPairs>;
    fn instrument_name(&self) -> Option<String>;
}

pub trait MessageExtendable {
    fn add_asset(&mut self,asset:&str);
    fn to_json(& self) -> anyhow::Result<String>;
}
/// For global usage;
lazy_static! {
    pub static ref EXCHANGES: Mutex<HashMap<&'static str, ExchangePairs<'static>>> = {
        let mut map = HashMap::new();
        let deribit_message = DeribitInitMessage::default();
        let deribit_name = "deribit";
        let okex_name = "okex";
        let okex_message = OkexInitMessage::default();
        let okex_url = "wss://ws.okx.com:8443/ws/v5/public";
        let deribit_url = "wss://www.deribit.com/ws/api/v2";
        map.insert(
            deribit_name,
            ExchangePairs::Deribit(deribit_name,deribit_url, deribit_message),
        );
        map.insert(okex_name, ExchangePairs::Okex(okex_name,okex_url, okex_message));

        Mutex::new(map)
    };
}
