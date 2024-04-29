use std::thread::yield_now;

use crate::exchanges::{Returnable, EXCHANGES};
use crate::trading::{Order, OrderBook, TradeRequest};
use futures_util::{SinkExt, StreamExt};
use serde::de::DeserializeOwned;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
pub async fn run_order_book<T: Returnable + DeserializeOwned + std::fmt::Debug>(
    exchange: &str,
    asset: Option<&str>,
) -> anyhow::Result<()> {
    if let Some(exchange_ref) = EXCHANGES.lock().await.get_mut(exchange) {
        let response = exchange_ref.response();

        if let Some(asset_name) = asset {
            response.add_asset(asset_name);
        }

        println!("fetching from {:?} from exchange {exchange}", &asset);
        let json = response.to_json()?;
        let url = exchange_ref.get_url();

        let mut order_book = OrderBook::new(exchange);
        let (stream, _) = connect_async(url).await?;
        let (mut writer, mut reader) = stream.split();

        let msg = Message::Text(json);

        writer.send(msg).await?;

        loop {
            if let Some(Ok(Message::Text(message))) = reader.next().await {
                let json: T = serde_json::from_str(&message)?;

                if let (Some((asks, bids)), Some(instrument_name)) =
                    (json.asks_bids_pair(), json.instrument_name())
                {
                    order_book.add_asset(instrument_name.clone());
                    let ask_pairs = asks.iter();
                    let bids_pairs = bids.iter();
                    let mut bids_count = 0;
                    let mut ask_count = 0;
                    ask_pairs.for_each(|(asking_price, quantity)| {
                        let ask_order =
                            Order::new(*asking_price, *quantity as i32, TradeRequest::Ask);

                        order_book.add_order(ask_order, &instrument_name);
                        ask_count += 1;
                    });

                    bids_pairs.for_each(|(biding_price, bid_quantity)| {
                        let bid_order =
                            Order::new(*biding_price, *bid_quantity as i32, TradeRequest::Bid);
                        order_book.add_order(bid_order, &instrument_name);
                        bids_count += 1;
                    });

                    order_book.match_orders(&instrument_name, None);

                    println!("+{ask_count} asks and + {bids_count}")
                }
            }
        }
    }

    Ok(())
}
