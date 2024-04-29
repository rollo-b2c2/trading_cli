use ordered_float::OrderedFloat;
use tokio_tungstenite::tungstenite::http::request;

use crate::trading::{MatchedOrders, OrderStatus};

use super::{Order, PriceColumns, TradeRequest};
use crate::utils::{add_each, get_timestamp_ms, match_at_price_level};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::sync::{Arc, Mutex};
use std::vec;
//  Table columns ->  HashMap<asset (BTC-USD), { asks,bids, spread,orders }>
pub type OrderTable = HashMap<String, PriceColumns>;
#[derive(Debug)]
pub struct OrderBook<'a> {
    pub exchange: &'a str,
    pub asset_order_table: OrderTable,
}

impl<'a> OrderBook<'a> {
    pub fn show(&self) {
        // create a table with popular library here using to display PriceColums here
    }

    pub fn new(exchange_name: &'a str) -> Self {
        Self {
            exchange: exchange_name,
            asset_order_table: HashMap::default(),
        }
    }

    pub fn add_asset(&mut self, name: String) {
        if !self.asset_order_table.contains_key(&name) {
            self.asset_order_table.entry(name).or_default();
        }
    }

    pub fn add_order(&mut self, mut order: Order, asset: &str) {
        if let Some(mut table) = self.asset_order_table.get_mut(asset) {
            add_each(table, &mut order);
            table.orders.lock().unwrap().push_back(order);

            table.update_spread_and_mid_price();
        }
    }
}

// -----------------ORDER_MATCHING LOGIC--------------------------------------------------------- //

impl<'a> OrderBook<'a> {
    pub fn match_orders(
        &mut self,
        assets: &str,
        mut external_price_column: Option<&mut PriceColumns>,
    ) {
        // extend our assert_table with the other another one from a different exchange

        let table = self.asset_order_table.get_mut(assets);

        if let Some(asset_table) = table {
            if let Some(mut col) = external_price_column {
                asset_table.extend(col)
            }
            use TradeRequest::*;
            let mut orders_to_remove = Vec::new();
            asset_table
                .orders
                .lock()
                .unwrap()
                .iter_mut()
                .filter(|current| !current.is_completed())
                .for_each(|mut order| {
                    let mut remaining_qty = order.quantity;
                    let price: OrderedFloat<f32> = order.price.into();
                    match order.request {
                        Ask => {
                            let mut bids = asset_table.bids.iter_mut();

                            if let Some((mut x, holding)) = bids.next_back() {
                                while price <= *x {
                                    let order_map = asset_table.orders.clone();
                                    let (matched_qty, remove) =
                                        match_at_price_level(holding, &mut remaining_qty);

                                    orders_to_remove.extend(remove);

                                    if matched_qty != 0 {
                                        let matched_order =
                                            MatchedOrders::new(**x, matched_qty, self.exchange);
                                        order.filled_with.push(matched_order);
                                    }
                                    if let Some((a, _)) = bids.next_back() {
                                        x = a;
                                    } else {
                                        break;
                                    }
                                }
                            }
                        }
                        Bid => {
                            let mut asks = asset_table.asks.iter_mut();

                            if let Some((mut x, holding)) = asks.next() {
                                while price >= *x {
                                    let order_map = asset_table.orders.clone();
                                    let (matched_qty, remove) =
                                        match_at_price_level(holding, &mut remaining_qty);
                                    orders_to_remove.extend(remove);
                                    if matched_qty != 0 {
                                        let matched_order =
                                            MatchedOrders::new(**x, matched_qty, self.exchange);
                                        order.filled_with.push(matched_order);
                                    }
                                    if let Some((a, _)) = asks.next() {
                                        x = a;
                                    } else {
                                        break;
                                    }
                                }
                            }
                        }
                    }

                    let acc_qty = order.filled_with.iter().fold(0, |c, n| c + n.quantity);
                    // to get an arbitrage, if our price column is extended with another column from a different exchange and some of our trades are filled with other
                    let is_arbitrage = order
                        .filled_with
                        .iter()
                        .any(|curr| curr.exchange != self.exchange);
                    order.is_arbitrage = is_arbitrage;

                    if remaining_qty != 0 && remaining_qty < order.quantity {
                        order.status = OrderStatus::Partial;
                        order.remaining_qty = remaining_qty;
                        println!(
                            "{:#?}  partially completed with the following trade matches {:#?}",
                            order, order.filled_with
                        );
                    }
                    if remaining_qty == 0
                        && !order.filled_with.is_empty()
                        && acc_qty == order.quantity
                    {
                        // marked as completed
                        order.status = OrderStatus::Completed;
                        if order.is_arbitrage {
                            println!(" arbitrage detectd")
                        }

                        println!(
                            "{:#?}  Completed with the following trade matches {:#?}",
                            order, order.filled_with
                        );
                        // remove the existing value in place
                        let completed = std::mem::take(order);
                        asset_table.history.push_back(completed);
                    }
                });
            for order_to_remove in orders_to_remove {
                let mut lock = asset_table.orders.lock().unwrap();
                if let Some(index) = lock.iter().position(|o| o.id == order_to_remove) {
                    lock.remove(index);
                }
            }

            asset_table.update_spread_and_mid_price();
            // remove all used items
            asset_table
                .bids
                .retain(|_, holding| holding.total_quantity != 0);
            asset_table
                .asks
                .retain(|_, holding| holding.total_quantity != 0);

            asset_table
                .orders
                .lock()
                .unwrap()
                .retain(|ord| ord.quantity > 0)
        }
    }
}
