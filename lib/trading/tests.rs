#[cfg(test)]
mod order_book {
    use crate::trading::{Order, OrderBook, TradeRequest};

    #[test]
    fn match_trades() {
        let mut order_book = OrderBook::new("test");
        let mut orders = vec![
            Order::new(0.72, 30, TradeRequest::Ask),
            Order::new(0.73, 20, TradeRequest::Ask),
            Order::new(0.90, 50, TradeRequest::Bid),
        ];
        let asset = "BTC-GBP";
        order_book.add_asset(asset.to_owned());
        for order in orders {
            order_book.add_order(order, asset);
        }
        order_book.match_orders(asset, None);
        // both asks are completed with  higher bid of 90,

        let cols = order_book.asset_order_table.get_mut(asset).unwrap();

        assert_eq!(cols.history.len(), 2);
        assert!(cols.orders.lock().unwrap().iter().any(|c|c.is_partial_completed()))

        // the remaining gty of ask is 10
    }
}
