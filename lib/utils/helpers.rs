use ordered_float::OrderedFloat;
use serde::de::DeserializeOwned;

use crate::{
    exchanges::Returnable,
    trading::{
        CurrentHoldingPerPrice, Order, OrderBook, OrderStatus, PriceColumns, PriceRow, TradeRequest,
    },
};
/// a utility function for adding new orders to the OrderBook Columns
pub fn add_each(table: &mut PriceColumns, order: &mut Order) {
    let Order {
        id,
        is_arbitrage: _,
        status: _,
        price,
        request,
        quantity,
        filled_with: _,
        remaining_qty: _,
    } = order;
    let key = *price;

    use TradeRequest::*;
    let existing_holding = if *request == Ask {
        table.asks.get_mut(&key.into())
    } else {
        table.bids.get_mut(&key.into())
    };
    let total_amount = (*quantity as f32) * *price;
    let stored_order = MininalOrder::new(*id, *quantity, *price);
    match (request, existing_holding) {
        (&mut Ask, Some(holding)) => {
            update_existing_holding(holding, *quantity, total_amount, stored_order);
        }
        (&mut Ask, None) => {
            let new_holding = create_new_holding(*quantity, total_amount, stored_order);
            let key = *price;
            table.asks.insert(key.into(), new_holding);
        }

        (&mut Bid, Some(holding)) => {
            update_existing_holding(holding, *quantity, total_amount, stored_order);
        }

        _ => {
            let new_holding = create_new_holding(*quantity, total_amount, stored_order);
            let key = *price;
            table.bids.insert(key.into(), new_holding);
        }
    }
}

pub fn round(x: f32, decimals: u32) -> f32 {
    let y = 10i32.pow(decimals) as f32;
    (x * y).round() / y
}

/// if an order with the same price already exists, just update exisiting properities
fn update_existing_holding(
    holding: &mut CurrentHoldingPerPrice,
    quantity: i32,
    total_amount: f32,
    stored_order: MininalOrder,
) {
    holding.total_quantity += quantity;

    holding.total_amount += total_amount;
    let amount = holding.total_amount;
    holding.total_amount = round(amount, 3);
    holding.orders.push(stored_order)
}

fn create_new_holding(
    quantity: i32,
    total_amount: f32,
    current_order: MininalOrder,
) -> CurrentHoldingPerPrice {
    CurrentHoldingPerPrice {
        total_quantity: quantity,
        total_amount,
        orders: vec![current_order],
    }
}

use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};

use crate::trading::MininalOrder;
/// Generates a  unique timestamp  to use an id
pub fn get_timestamp_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis()
}

use std::collections::HashMap;
/// updates the quantity required to complete a trade at price level
pub fn match_at_price_level(
    current_holding: &mut CurrentHoldingPerPrice,
    incoming_order_qty: &mut i32,
) -> (i32, Vec<u128>) {
    let mut done_qty = 0;

    let mut orders_to_remove = vec![];

    current_holding.orders.iter_mut().for_each(|order| {
        if order.qty <= *incoming_order_qty {
            *incoming_order_qty -= order.qty;
            done_qty += order.qty;
            let reduced_amount = done_qty as f32 * order.price;
            current_holding.total_quantity -= done_qty;
            current_holding.total_amount -= reduced_amount;
            order.qty = 0;
            orders_to_remove.push(order.id);
        } else {
            order.qty -= *incoming_order_qty;
            done_qty += *incoming_order_qty;
            *incoming_order_qty = 0
        }
    });

    current_holding.orders.retain(|x| x.qty != 0);

    (done_qty, orders_to_remove)
}
