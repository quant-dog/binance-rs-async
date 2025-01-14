#[macro_use]
extern crate tokio;

use binance::api::*;
use binance::futures::userstream::*;
use binance::futures::websockets::*;
use binance::futures::ws_model::{CombinedStreamEvent, FuturesWebsocketEvent, FuturesWebsocketEventUntag};

use binance::websockets::partial_book_depth_stream;
use futures::future::BoxFuture;
use futures::stream::StreamExt;
use serde_json::from_str;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::RwLock;
use tokio::sync::mpsc::UnboundedSender;
use tokio_tungstenite::tungstenite::Message;

use tracing::{debug, info};
use tracing_subscriber;

use dotenv;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    tracing_subscriber::fmt::init();

    let (logger_tx, mut logger_rx) = tokio::sync::mpsc::unbounded_channel::<FuturesWebsocketEvent>();
    let (close_tx, mut close_rx) = tokio::sync::mpsc::unbounded_channel::<bool>();
    let wait_loop = tokio::spawn(async move {
        'hello: loop {
            select! {
                _event = logger_rx.recv() => {
                    // println!("{event:?}")
                },
                _ = close_rx.recv() => break 'hello
            }
        }
    });

    // private api
    // user_stream().await;
    // user_stream_websocket().await;

    // public api
    let streams: Vec<BoxFuture<'static, ()>> = vec![
        // Box::pin(market_websocket(logger_tx.clone())),
        // Box::pin(kline_websocket(logger_tx.clone())),
        // Box::pin(all_trades_websocket(logger_tx.clone())),
        // Box::pin(last_price(logger_tx.clone())),
        // Box::pin(book_ticker(logger_tx.clone())),
        Box::pin(combined_orderbook(logger_tx.clone())),
        // Box::pin(custom_event_loop(logger_tx)),
    ];

    for stream in streams {
        tokio::spawn(stream);
    }

    select! {
        _ = wait_loop => { println!("Finished!") }
        _ = tokio::signal::ctrl_c() => {
            println!("\nClosing websocket stream...");
            close_tx.send(true).unwrap();
        }
    }
}

#[allow(dead_code)]
async fn user_stream() {
    let api_key = std::env::var("BINANCE_API_KEY").ok();
    // let secret_key = std::env::var("BINANCE_API_SECRET_KEY").ok();

    let user_stream: FuturesUserStream = Binance::new(api_key.clone(), None);

    if let Ok(answer) = user_stream.start().await {
        println!("Data Stream Started ...");
        let listen_key = answer.listen_key;

        // loop {
        match user_stream.keep_alive(&listen_key).await {
            Ok(msg) => println!("Keepalive user data stream: {msg:?}"),
            Err(e) => println!("Error: {e}"),
        }

        //     tokio::time::sleep(tokio::time::Duration::from_secs(20)).await;
        // }

        #[allow(dead_code)]
        match user_stream.close(&listen_key).await {
            Ok(msg) => println!("Close user data stream: {msg:?}"),
            Err(e) => println!("Error: {e}"),
        }
    } else {
        println!("Not able to start an User Stream (Check your API_KEY)");
    }
}

#[allow(dead_code)]
async fn user_stream_websocket() {
    let keep_running = AtomicBool::new(true); // Used to control the event loop

    let api_key = std::env::var("BINANCE_API_KEY").ok();
    // let secret_key = std::env::var("BINANCE_API_SECRET_KEY").ok();

    let user_stream: FuturesUserStream = Binance::new(api_key, None);

    if let Ok(answer) = user_stream.start().await {
        let listen_key = answer.listen_key;

        let mut web_socket: FuturesWebSockets<'_, FuturesWebsocketEvent> = FuturesWebSockets::new(
            |event: FuturesWebsocketEvent| {

                match event {
                    FuturesWebsocketEvent::OrderTradeUpdate(trade) => {
                        let data = trade.order_trade;
                        println!(
                            "Symbol: {}, Side: {:?}, Qty: {:?}, Price: {}, AvgPrice: {}, Execution Type: {:?}, Order Status: {:?}",
                            data.symbol, data.side, data.qty, data.price, data.avg_price, data.execution_type, data.order_status
                        );
                    },
                    FuturesWebsocketEvent::AccountUpdate(account) => {
                        let data = account.update_data;
                        for balance in data.balances {
                            println!("{:?}", balance);    
                        }
                        for position in data.positions {
                            println!("{:?}", position);    
                        }
                    },

                    _ => {}

                }

                Ok(())
            },
        );

        web_socket.connect(&listen_key).await.unwrap(); // check error
        if let Err(e) = web_socket.event_loop(&keep_running).await {
            println!("Error: {e}");
        }
        user_stream.close(&listen_key).await.unwrap();
        web_socket.disconnect().await.unwrap();
        println!("Userstrem closed and disconnected");
    } else {
        println!("Not able to start an User Stream (Check your API_KEY)");
    }
}

#[allow(dead_code)]
async fn market_websocket(logger_tx: UnboundedSender<FuturesWebsocketEvent>) {
    let keep_running = AtomicBool::new(true); // Used to control the event loop
    // let agg_trade: String = agg_trade_stream("dydxusdt");
    let partial_order_book = partial_book_depth_stream("btcusdt", 5, 100);
    let mut web_socket: FuturesWebSockets<'_, FuturesWebsocketEvent> =
        FuturesWebSockets::new(|event: FuturesWebsocketEvent| {
            

            if let Err(e) = logger_tx.send(event.clone()) {
                println!("Failed to send more messages, channel shutdown {e:?}");
            }
            match event {
                FuturesWebsocketEvent::Trade(trade) => {
                    println!("Symbol: {}, price: {}, qty: {}", 
                    trade.symbol, 
                    trade.price, 
                    trade.qty);
                }
                FuturesWebsocketEvent::DepthOrderBook(depth_order_book) => {
                    println!(
                        "Symbol: {}, bid1: {:?}, ask1: {:?}",
                        depth_order_book.symbol, 
                        depth_order_book.bids[0], 
                        depth_order_book.asks[0]
                    );
                }
                _ => (),
            };

            Ok(())
        });

    web_socket.connect(&partial_order_book).await.unwrap(); // check error
    if let Err(e) = web_socket.event_loop(&keep_running).await {
        println!("Error: {e}");
    }
    web_socket.disconnect().await.unwrap();
    println!("disconnected");
}

#[allow(dead_code)]
async fn all_trades_websocket(logger_tx: UnboundedSender<FuturesWebsocketEvent>) {
    let keep_running = AtomicBool::new(true); // Used to control the event loop
    let agg_trade = all_ticker_stream();
    // NB: you may not ask for both arrays type streams and object type streams at the same time, this holds true in binance connections anyways
    // You cannot connect to multiple things for a single socket
    let mut web_socket: FuturesWebSockets<'_, Vec<FuturesWebsocketEvent>> =
        FuturesWebSockets::new(|events: Vec<FuturesWebsocketEvent>| {
            for tick_events in events {
                logger_tx.send(tick_events.clone()).unwrap();
                if let FuturesWebsocketEvent::DayTicker(tick_event) = tick_events {
                    println!(
                        "Symbol: {}, price: {}, qty: {}",
                        tick_event.symbol, tick_event.best_bid, tick_event.best_bid_qty
                    );
                }
            }

            Ok(())
        });

    web_socket.connect(agg_trade).await.unwrap(); // check error
    if let Err(e) = web_socket.event_loop(&keep_running).await {
        println!("Error: {e}");
    }
    web_socket.disconnect().await.unwrap();
    println!("disconnected");
}

#[allow(dead_code)]
async fn kline_websocket(logger_tx: UnboundedSender<FuturesWebsocketEvent>) {
    let keep_running = AtomicBool::new(true);
    let kline = kline_stream("ethbtc", "1m");
    let mut web_socket: FuturesWebSockets<'_, FuturesWebsocketEvent> =
        FuturesWebSockets::new(|event: FuturesWebsocketEvent| {
            logger_tx.send(event.clone()).unwrap();
            if let FuturesWebsocketEvent::Kline(kline_event) = event {
                println!(
                    "Symbol: {}, high: {}, low: {}",
                    kline_event.kline.symbol, kline_event.kline.low, kline_event.kline.high
                );
            }

            Ok(())
        });

    web_socket.connect(&kline).await.unwrap(); // check error
    if let Err(e) = web_socket.event_loop(&keep_running).await {
        println!("Error: {e}");
    }
    web_socket.disconnect().await.unwrap();
    println!("disconnected");
}

#[allow(dead_code)]
async fn last_price(logger_tx: UnboundedSender<FuturesWebsocketEvent>) {
    let keep_running = AtomicBool::new(true);
    let all_ticker = all_ticker_stream();
    let btcusdt: RwLock<f32> = RwLock::new("0".parse().unwrap());

    let mut web_socket: FuturesWebSockets<'_, Vec<FuturesWebsocketEvent>> =
        FuturesWebSockets::new(|events: Vec<FuturesWebsocketEvent>| {
            for tick_events in events {
                logger_tx.send(tick_events.clone()).unwrap();
                if let FuturesWebsocketEvent::DayTicker(tick_event) = tick_events {
                    if tick_event.symbol == "BTCUSDT" {
                        let mut btcusdt = btcusdt.write().unwrap();
                        *btcusdt = tick_event.average_price.parse::<f32>().unwrap();
                        let btcusdt_close: f32 = tick_event.current_close.parse().unwrap();
                        println!("{btcusdt} - {btcusdt_close}");

                        if btcusdt_close as i32 == 7000 {
                            // Break the event loop
                            keep_running.store(false, Ordering::Relaxed);
                        }
                    }
                }
            }

            Ok(())
        });

    web_socket.connect(all_ticker).await.unwrap(); // check error
    if let Err(e) = web_socket.event_loop(&keep_running).await {
        println!("Error: {e}");
    }
    web_socket.disconnect().await.unwrap();
    println!("disconnected");
}

#[allow(dead_code)]
async fn book_ticker(logger_tx: UnboundedSender<FuturesWebsocketEvent>) {
    let keep_running = AtomicBool::new(true);
    let book_ticker: String = book_ticker_stream("btcusdt");

    let mut web_socket: FuturesWebSockets<'_, FuturesWebsocketEventUntag> =
        FuturesWebSockets::new(|events: FuturesWebsocketEventUntag| {
            if let FuturesWebsocketEventUntag::FuturesWebsocketEvent(we) = &events {
                logger_tx.send(we.clone()).unwrap();
            }
            if let FuturesWebsocketEventUntag::BookTicker(tick_event) = events {
                println!("{tick_event:?}")
            }
            Ok(())
        });

    web_socket.connect(&book_ticker).await.unwrap(); // check error
    if let Err(e) = web_socket.event_loop(&keep_running).await {
        println!("Error: {e}");
    }
    web_socket.disconnect().await.unwrap();
    println!("disconnected");
}

#[allow(dead_code)]
async fn combined_orderbook(logger_tx: UnboundedSender<FuturesWebsocketEvent>) {
    
    let keep_running = AtomicBool::new(true);
    let streams: Vec<String> = vec!["btcusdt", "ethusdt"]
        .into_iter()
        .map(|symbol| partial_book_depth_stream(symbol, 5, 500))
        .collect();

        let mut web_socket =
        FuturesWebSockets::new(|event: CombinedStreamEvent<FuturesWebsocketEvent>| {
            let data = event.data;
            let symbol: String = event.stream.split_once('@').unwrap().0.to_string().to_uppercase();
            if let FuturesWebsocketEvent::DepthOrderBook(orderbook) = data {
                let (bid1, ask1) = (orderbook.bids[0].price, orderbook.asks[0].price);
                info!("orderbook:{symbol} {bid1}/{ask1}");
            } else {
                debug!("unknown event with symbol: -- {symbol} --");
            }

            Ok(())
        });

    web_socket.connect_multiple(streams).await.unwrap(); // check error
    if let Err(e) = web_socket.event_loop(&keep_running).await {
        println!("Error: {e}");
    }
    web_socket.disconnect().await.unwrap();
    println!("disconnected");
}

#[allow(dead_code)]
async fn custom_event_loop(logger_tx: UnboundedSender<FuturesWebsocketEvent>) {
    let streams: Vec<String> = vec!["btcusdt", "ethusdt"]
        .into_iter()
        .map(|symbol| partial_book_depth_stream(symbol, 5, 100))
        .collect();
    let mut web_socket: FuturesWebSockets<'_, CombinedStreamEvent<_>> =
        FuturesWebSockets::new(|event: CombinedStreamEvent<FuturesWebsocketEventUntag>| {
            debug!("event: {event:?}");
            if let FuturesWebsocketEventUntag::FuturesWebsocketEvent(we) = &event.data {
                logger_tx.send(we.clone()).unwrap();
            }
            let data = event.data;
            if let FuturesWebsocketEventUntag::Orderbook(orderbook) = data {
                println!("{orderbook:?}")
            }
            Ok(())
        });
    web_socket.connect_multiple(streams).await.unwrap(); // check error
    loop {
        if let Some((ref mut socket, _)) = web_socket.socket {
            if let Ok(message) = socket.next().await.unwrap() {
                match message {
                    Message::Text(msg) => {
                        if msg.is_empty() {
                            continue;
                        }
                        let event: CombinedStreamEvent<FuturesWebsocketEventUntag> = from_str(msg.as_str()).unwrap();
                        eprintln!("event = {event:?}");
                    }
                    Message::Ping(_) | Message::Pong(_) | Message::Binary(_) | Message::Frame(_) => {}
                    Message::Close(e) => {
                        eprintln!("closed stream = {e:?}");
                        break;
                    }
                }
            }
        }
    }
}
