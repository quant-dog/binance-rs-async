#![allow(unused)]

use crate::rest_model::{string_or_bool, string_or_float, string_or_float_opt, string_or_u64};

pub use crate::futures::rest_model::{
    Asks, Bids, OrderBook, OrderSide, OrderStatus, OrderType, PositionSide, Success, TimeInForce, WorkingType,
};

pub use crate::ws_model::{
    AccountPositionUpdate, BalanceUpdate, BookTickerEvent, CombinedStreamEvent, DayTickerEvent,
    DepthOrderBookEvent, EventBalance, Kline, KlineEvent, MiniDayTickerEvent, OrderListTransaction, OrderListUpdate,
    OrderUpdate, QueryResult, TradeEvent, TradesEvent, WebsocketEvent, WebsocketEventUntag,
};

///
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FuturesWebsocketEventUntag {
    FuturesWebsocketEvent(FuturesWebsocketEvent),
    Orderbook(Box<OrderBook>),
    BookTicker(Box<BookTickerEvent>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "e")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FuturesWebsocketEvent {
    #[serde(alias = "aggTrade")]
    AggTrade(Box<TradesEvent>),

    #[serde(alias = "trade")]
    Trade(Box<TradeEvent>),
    
    #[serde(alias = "kline")]
    Kline(Box<KlineEvent>),
    
    #[serde(alias = "24hrTicker")]
    DayTicker(Box<DayTickerEvent>),
    
    #[serde(alias = "24hrMiniTicker")]
    DayMiniTicker(Box<MiniDayTickerEvent>),
    
    #[serde(alias = "depthUpdate")]
    DepthOrderBook(Box<DepthOrderBookEvent>),
    
    #[serde(alias = "outboundAccountPosition")]
    AccountPositionUpdate(Box<AccountPositionUpdate>),
    
    #[serde(alias = "balanceUpdate")]
    BalanceUpdate(Box<BalanceUpdate>),
    
    #[serde(alias = "executionReport")]
    OrderUpdate(Box<OrderUpdate>),
    
    #[serde(alias = "listStatus")]
    ListOrderUpdate(Box<OrderListUpdate>),
    
    // futures specific
    #[serde(alias = "orderTradeUpdate")]
    OrderTradeUpdate(Box<OrderTradeUpdate>),
    
    #[serde(alias = "accountUpdate")]
    AccountUpdate(Box<AccountUpdate>),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OrderTradeUpdate {
    #[serde(rename = "T")]
    pub transact_time: u64,

    #[serde(rename = "E")]
    pub event_time: u64,

    #[serde(rename = "o")]
    pub order_trade: OrderTradeUpdateInner,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OrderTradeUpdateInner {
    #[serde(rename = "s")]
    pub symbol: String,

    #[serde(rename = "c")]
    pub client_order_id: Option<String>,

    #[serde(rename = "S")]
    pub side: OrderSide,

    #[serde(rename = "o")]
    pub order_type: OrderType,

    #[serde(rename = "f")]
    pub time_in_force: TimeInForce,

    #[serde(rename = "q")]
    #[serde(with = "string_or_float")]
    pub qty: f64,

    #[serde(rename = "p")]
    #[serde(with = "string_or_float")]
    pub price: f64,

    #[serde(rename = "ap")]
    #[serde(with = "string_or_float")]
    pub avg_price: f64,

    #[serde(rename = "sp")]
    #[serde(with = "string_or_float")]
    pub stop_price: f64,

    #[serde(rename = "x")]
    pub execution_type: OrderStatus,

    #[serde(rename = "X")]
    pub order_status: OrderStatus,

    #[serde(rename = "i")]
    pub order_id: u64,

    #[serde(rename = "l")]
    #[serde(with = "string_or_float")]
    pub qty_last_executed: f64,

    #[serde(rename = "z")]
    #[serde(with = "string_or_float")]
    pub cumulative_filled_qty: f64,

    #[serde(rename = "L")]
    #[serde(with = "string_or_float")]
    pub last_executed_price: f64,

    #[serde(rename = "n")]
    #[serde(with = "string_or_float")]
    pub commission: f64,

    #[serde(rename = "N")]
    pub commission_asset: Option<String>,

    #[serde(rename = "T")]
    pub trade_order_time: u64,

    #[serde(rename = "t")]
    pub trade_id: i64,

    #[serde(rename = "b")]
    #[serde(with = "string_or_float")]
    pub bids_notional: f64,

    #[serde(rename = "a")]
    #[serde(with = "string_or_float")]
    pub asks_notional: f64,

    #[serde(rename = "m")]
    pub is_maker: bool,

    #[serde(rename = "R")]
    pub is_reduce: bool,

    #[serde(rename = "wt")]
    pub sp_working_type: WorkingType,

    #[serde(rename = "ot")]
    pub orig_type: OrderType,

    #[serde(rename = "ps")]
    pub position_side: PositionSide,

    #[serde(rename = "cp")]
    pub is_push_conditional: bool,

    // #[serde(rename = "AP")]
    // #[serde(with = "string_or_float")]
    // pub activation_price: f64,

    #[serde(rename = "rp")]
    #[serde(with = "string_or_float")]
    pub realized_profit_ignore: f64,

    // #[serde(rename = "cr")]
    // #[serde(with = "string_or_float")]
    // pub callback_rate: f64,

    #[serde(rename = "pP")]
    pub pp_ignore: bool,

    #[serde(rename = "si")]
    pub si_ignore: i64,

    #[serde(rename = "ss")]
    pub ss_ignore: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AccountUpdate {
    #[serde(rename = "T")]
    pub transact_time: u64,

    #[serde(rename = "E")]
    pub event_time: u64,

    #[serde(rename = "a")]
    pub update_data: AccountUpdateDataInner,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "UPPERCASE")]
pub enum EventReasonType {
    Order,
    AutoExchange,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AccountUpdateDataInner {
    #[serde(rename = "m")]
    pub event_reason_type: EventReasonType,

    #[serde(rename = "B")]
    pub balances: Vec<AccountUpdateBalancesInner>,
    
    #[serde(rename = "P")]
    pub positions: Vec<AccountUpdatePositionsInner>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AccountUpdateBalancesInner {
    #[serde(rename = "a")]
    pub asset: String,

    #[serde(rename = "wb")]
    #[serde(with = "string_or_float")]
    pub wallet_balance: f64,

    #[serde(rename = "cw")]
    #[serde(with = "string_or_float")]
    pub cross_wallet_balance: f64,

    #[serde(rename = "bc")]
    #[serde(with = "string_or_float")]
    pub balance_change_ex: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AccountUpdatePositionsInner {
    #[serde(rename = "s")]
    pub symbol: String,

    #[serde(rename = "pa")]
    #[serde(with = "string_or_float")]
    pub position_amount: f64,

    #[serde(rename = "ep")]
    #[serde(with = "string_or_float")]
    pub early_price: f64,

    #[serde(rename = "cr")]
    #[serde(with = "string_or_float")]
    pub accumulated_realized: f64,

    #[serde(rename = "up")]
    #[serde(with = "string_or_float")]
    pub unrealized_pnl: f64,

    #[serde(rename = "mt")]
    pub margin_type: String,

    #[serde(rename = "iw")]
    #[serde(with = "string_or_float")]
    pub isolated_wallet: f64,

    #[serde(rename = "ps")]
    pub position_side: PositionSide,
}
