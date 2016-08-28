
extern crate regex;
#[macro_use]
extern crate lazy_static;

use regex::{Regex,Captures};
#[allow(unused_imports)]
use std::mem::size_of;





//
//Define regex
//
lazy_static! {
static ref CAPTUREVALUE: Regex = Regex::new(
r#""*(\d{1,6})\.?(\d{1,12})?"#).unwrap();

static ref CAPTUREUUID: Regex = Regex::new(
r#"([a-f\d]{8})-([a-f\d]{4})-([a-f\d]{4})-([a-f\d]{4})-([a-f\d]{12})"#).unwrap();

//Open order
static ref CAPTURE_OPEN_ORDER: Regex = Regex::new(
r#".*"type":"open",.*"side":"(buy|sell)","price":(null|"[\d\.]+"),"order_id":"([a-z\d-]+)","remaining_size":(null|"[\d\.]+")"#
).unwrap();

//capture a done order
static ref CAPTURE_DONE_ORDER: Regex = Regex::new(
r#".*type":"done","order_type":"limit","side":"(buy|sell)".*"order_id":"([a-z\d-]+)""#
).unwrap();

//capture a match order
static ref CAPTURE_MATCH_ORDER: Regex = Regex::new(
r#".*"type":"match","sequence":.*"taker_order_id":"([a-f\d-]+)","side":"(buy|sell)","size":(null|"[\d\.]+"),"price":(null|"[\d\.]+"),"#
).unwrap();

//match a received order
static ref IGNORE_PACKET: Regex = Regex::new(
r#"\{"type":"change|received".*}"#
).unwrap();

static ref IGNORE_PACKET_2: Regex = Regex::new(
r#"\{"type":"done","order_type":"market".*}"#
).unwrap();

//match an error packet
static ref ERROR_PACKET: Regex = Regex::new(
r#"\{"type":"closing"\}"#
).unwrap();

//matching a reset packet
static ref RESET_PACKET: Regex = Regex::new(
r#".*"type":"reset".*"#
).unwrap();
}

//
//Represents a value from
//99,999,999.9999999999
//          10000000000
//
#[derive(Clone,Copy,PartialEq,Eq,PartialOrd,Ord,Debug)]
pub struct Value {
    data: u64
}
impl Value {

    //
    //Create a new value
    //
    pub fn new<'a>(buffer: &'a str) -> Option<Value> {
        if buffer == "null" {
            return Some(Value{ data: 0});
        }
        //look for a match
        let caps: Captures<'a> = match CAPTUREVALUE.captures(buffer) {
            Option::None => return None,
            Option::Some(x) => x
        };
        //get top value
        let top = match caps.at(1) {
            Option::None => return None,
            Option::Some(upper) => match u64::from_str_radix(upper, 10) {
                Err(_) => return None,
                Ok(x) => x * 10000000000u64
            }
        };
        //get bottom value
        match caps.at(2) {
            Option::None => Some(Value{data: top}),
            Option::Some(bottom) => match u64::from_str_radix(bottom, 10) {
                //bitch at me for writing bad code idgaf
                Ok(bot) => match bottom.len() {
                    1  => Some(Value{data: top+(bot*1000000000u64)}),
                    2  => Some(Value{data: top+(bot*100000000u64)}),
                    3  => Some(Value{data: top+(bot*10000000u64)}),
                    4  => Some(Value{data: top+(bot*1000000u64)}),
                    5  => Some(Value{data: top+(bot*100000u64)}),
                    6  => Some(Value{data: top+(bot*10000u64)}),
                    7  => Some(Value{data: top+(bot*1000u64)}),
                    8  => Some(Value{data: top+(bot*100u64)}),
                    9  => Some(Value{data: top+(bot*10u64)}),
                    10 => Some(Value{data: top+bot}),
                    _ => unreachable!()
                },
                Err(_) => None
            }
        }
    }

    //
    //Short cut for a very small value
    //
    pub fn zero() -> Value {
        Value{
            data: 0
        }
    }

    //
    //Short cut for a very large value
    //
    pub fn max() -> Value {
        Value {
            data: u64::max_value()
        }
    }

    //
    //Convert to a float 64 (loss of precision will occur)
    //
    pub fn to_f64(&self) -> f64 {
        self.data as f64 / 10000000000f64
    }

    //
    //Convert from a float
    //
    pub fn from_f64(x: f64) -> Value {
        Value {
            data: (x*10000000000f64) as u64
        }
    }
}
unsafe impl Send for Value {

}
#[test]
fn test_value() {
    let dut_str = "5872.75856521";
    let dut_val = Value::new(dut_str).unwrap();
    assert_eq!(dut_val.data, 58727585652100u64);
    assert_eq!(dut_val.to_f64(), 5872.75856521f64);

    let dut_str = "0";
    let dut_val = Value::new(dut_str).unwrap();
    assert_eq!(dut_val.data, 0);
    assert_eq!(dut_val.to_f64(), 0f64);

    let dut_str = "null";
    let dut_val = Value::new(dut_str).unwrap();
    assert_eq!(dut_val.data, 0);
    assert_eq!(dut_val.to_f64(), 0f64);

    let dut_str = "20253.3278928455";
    let dut_val = Value::new(dut_str).unwrap();
    assert_eq!(dut_val.data, 202533278928455);

    let dut_str = "15521.6044582977";
    let dut_val = Value::new(dut_str).unwrap();
    assert_eq!(dut_val.data,155216044582977);

    let dut_str = "\"15521.6044582977\"";
    let dut_val = Value::new(dut_str).unwrap();
    assert_eq!(dut_val.data,155216044582977);

    let dut_str = "48.08146";
    let dut_val = Value::new(dut_str).unwrap();
    assert_eq!(dut_val.data,480814600000);

    assert_eq!( 8, size_of::<Value>() );
}

//
//Represents a UUID
//
#[derive(Clone,Copy,PartialEq,Eq,PartialOrd,Ord,Debug)]
pub struct UUID {
    data_a: u64,
    data_b: u64
}
impl UUID {

    //
    //Create a new UUID from a string
    //
    pub fn new<'a>(buffer: &'a str) -> Option<UUID> {
        //extract captures
        let caps: Captures<'a> = match CAPTUREUUID.captures(buffer){
            Option::None => return None,
            Option::Some(x) => x
        };
        let a = match caps.at(1) {
            Option::None => return None,
            Option::Some(x) => match u32::from_str_radix(x,16){
                Err(_) => return None,
                Ok(x) => (x as u64).wrapping_shl(32)
            }
        };
        let b = match caps.at(2) {
            Option::None => return None,
            Option::Some(x) => match u16::from_str_radix(x,16) {
                Err(_) => return None,
                Ok(x) => (x as u64).wrapping_shl(16)
            }
        };
        let c = match caps.at(3) {
            Option::None => return None,
            Option::Some(x) => match u16::from_str_radix(x,16) {
                Err(_) => return None,
                Ok(x) => x as u64
            }
        };
        let d = match caps.at(4) {
            Option::None => return None,
            Option::Some(x) => match u16::from_str_radix(x,16) {
                Err(_) => return None,
                Ok(x) => (x as u64).wrapping_shl(48)
            }
        };
        let e = match caps.at(5) {
            Option::None => return None,
            Option::Some(x) => match u64::from_str_radix(x,16) {
                Err(_) => return None,
                Ok(x) => x as u64
            }
        };
        //return a value
        Some(UUID {
            data_a: a.wrapping_add(b.wrapping_add(c)),
            data_b: d.wrapping_add(e)
        })
    }
}
unsafe impl Send for UUID {

}
#[test]
fn test_uuid() {
    let dut_str = "00000000-0000-0000-0000-000000000000";
    let dut_uuid = UUID::new(dut_str).unwrap();
    assert_eq!( dut_uuid.data_a, 0u64);
    assert_eq!( dut_uuid.data_b, 0u64);

    let dut_str = "00000000-0000-0000-0000-000000000001";
    let dut_uuid = UUID::new(dut_str).unwrap();
    assert_eq!( dut_uuid.data_a, 0u64);
    assert_eq!( dut_uuid.data_b, 1u64);

    let dut_str = "00000000-0000-000f-0000-00000000000a";
    let dut_uuid = UUID::new(dut_str).unwrap();
    assert_eq!( dut_uuid.data_a, 15u64);
    assert_eq!( dut_uuid.data_b, 10u64);

    let dut_str = "ffff307e-6c5e-4cfc-bed5-6be2a70b8b42";
    let _ = UUID::new(dut_str).unwrap();

    assert_eq!( 16, size_of::<UUID>() );
}


//
//Enum that determines buy/sell
//
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Side{
    Sell,
    Buy
}
impl Side {
    pub fn new(buffer:&str) -> Option<Side> {
        if buffer == "sell" {
            Some(Side::Sell)
        } else if buffer == "buy" {
            Some(Side::Buy)
        } else {
            None
        }
    }
}
unsafe impl Send for Side {
}

//
//This represents a single order
//
#[derive(Clone)]
pub struct Order {
    pub id: UUID,
    pub price: Value,
    pub size: Value
}
unsafe impl Send for Order {
}
impl PartialEq for Order {
    fn eq(&self,other:&Order) -> bool {
        self.id == other.id
    }
    fn ne(&self,other:&Order) -> bool {
        self.id != other.id
    }
}
impl Eq for Order {

}
impl PartialOrd for Order {
    fn partial_cmp(&self, other: &Order) -> Option<::std::cmp::Ordering> {
        self.id.partial_cmp( &other.id )
    }
}
impl Ord for Order {
    fn cmp(&self,other: &Order) -> ::std::cmp::Ordering {
        self.id.cmp( &other.id )
    }
}

//
//This is a high level representation of a transaction
//
//          This OPENS an order on the book
//      Order::Open(<OrderID>,<Buy/Sell>,<Price>,<Size>)
//
//          This removes an order from the book
//      Order::Done(<OrderID>,<Buy/Sell>)
//
//          This changes an order on the books
//      Order::Modify(<OrderID>,<Buy/Sell>,<Delta Size>)
//
//          This empties the table
//      Order::Reset
//
//          This ends the program
//      Order::Error
//
#[derive(Clone,Debug,PartialEq,Eq)]
pub enum OrderBookOp {
    Open(UUID,Side,Value,Value),
    Done(UUID,Side),
    Modify(UUID,Side,Value),
    Reset,
    Error,
    Ignored
}
unsafe impl Send for OrderBookOp {

}

pub fn OpenOrder(packet: &str) -> Option<OrderBookOp> {
    match CAPTURE_OPEN_ORDER.captures(packet) {
        Option::None => None,
        Option::Some(caps) => match caps.at(1) {
            Option::None => None,
            Option::Some(buy_sell) => match Side::new(buy_sell) {
                Option::None => None,
                Option::Some(side) => match caps.at(2) {
                    Option::None => None,
                    Option::Some(price_str) => match Value::new(price_str) {
                        Option::None => None,
                        Option::Some(value_price) => match caps.at(3) {
                            Option::None => None,
                            Option::Some(uuid_str) => match UUID::new(uuid_str) {
                                Option::None => None,
                                Option::Some(uuid) => match caps.at(4) {
                                    Option::None => None,
                                    Option::Some(size_str) => match Value::new(size_str) {
                                        Option::None => None,
                                        Option::Some(size_val) => Some(OrderBookOp::Open(uuid,side,value_price,size_val))
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}


pub fn DoneOrder(packet: &str) -> Option<OrderBookOp> {
    match CAPTURE_DONE_ORDER.captures(packet) {
        Option::None => None,
        Option::Some(caps) => match caps.at(1) {
            Option::None => None,
            Option::Some(side_str) => match caps.at(2) {
                Option::None => None,
                Option::Some(uuid_str) => match Side::new(side_str) {
                    Option::None => None,
                    Option::Some(side) => match UUID::new(uuid_str) {
                        Option::None => None,
                        Option::Some(uuid) => Some(OrderBookOp::Done(uuid,side))
                    }
                }
            }
        }
    }
}

pub fn ModifyOrder(packet: &str) -> Option<OrderBookOp> {
    match CAPTURE_MATCH_ORDER.captures(packet) {
        Option::None => None,
        Option::Some(caps) => match caps.at(1) {
            Option::None => None,
            Option::Some(uuid_str) => match caps.at(2) {
                Option::None => None,
                Option::Some(side_str) => match caps.at(3) {
                    Option::None => None,
                    Option::Some(size_str) => match UUID::new(uuid_str) {
                        Option::None => None,
                        Option::Some(uuid) => match Side::new(side_str) {
                            Option::None => None,
                            Option::Some(side) => match Value::new(size_str) {
                                Option::None => None,
                                Option::Some(size) => Some(OrderBookOp::Modify(uuid,side,size))
                            }
                        }
                    }
                }
            }
        }
    }
}

pub fn MakeOrderBookOp(packet: &str)-> Option<OrderBookOp> {
    if IGNORE_PACKET.is_match(packet) || IGNORE_PACKET_2.is_match(packet) {
        return Some(OrderBookOp::Ignored);
    } else if ERROR_PACKET.is_match(packet){
        return Some(OrderBookOp::Error);
    } else if RESET_PACKET.is_match(packet) {
        return Some(OrderBookOp::Reset);
    }
    match OpenOrder(packet) {
        Option::Some(x) => Some(x),
        Option::None => match DoneOrder(packet) {
            Option::Some(x) => Some(x),
            Option::None => match ModifyOrder(packet) {
                Option::Some(x) => Some(x),
                Option::None => None
            }
        }
    }
}


#[test]
fn test_packet_reader() {
    let reset_packet = r#"{"type":"reset"}"#;
    let op = MakeOrderBookOp(reset_packet).unwrap();
    assert_eq!(op,OrderBookOp::Reset);

    let reset_packet = r#"{"type":"closing"}"#;
    let op = MakeOrderBookOp(reset_packet).unwrap();
    assert_eq!(op,OrderBookOp::Error);

    let received_packet = r#"{"type":"received","sequence":1415928821,"order_id":"110f289b-359e-4151-96ac-8b81d05e4fc8","order_type":"limit","size":"0.125","price":"582.45","side":"buy","funds":null,"product_id":"BTC-USD","time":"2016-08-23T17:31:55.504125Z"}"#;
    let op = MakeOrderBookOp(received_packet).unwrap();
    assert_eq!(op, OrderBookOp::Ignored);

    let received_packet = r#"{"type":"received","sequence":1415928851,"order_id":"2d99ff0c-2b5a-4729-be62-7249fb7990e6","order_type":"market","size":"0.17","price":null,"side":"buy","funds":"99.0233","product_id":"BTC-USD","time":"2016-08-23T17:32:03.042217Z"}"#;
    let op = MakeOrderBookOp(received_packet).unwrap();
    assert_eq!(op, OrderBookOp::Ignored);

    let closed_market = r#"{"type":"done","order_type":"market","side":"buy","sequence":1415931125,"order_id":"96dbdcd4-d0d7-482a-aea5-44014ce4065e","reason":"filled","product_id":"BTC-USD","time":"2016-08-23T17:32:31.701227Z"}"#;
    let op = MakeOrderBookOp(closed_market).unwrap();
    assert_eq!(op, OrderBookOp::Ignored);

    let open_packet = r#"{"type":"open","sequence":1415928822,"side":"buy","price":"582.45","order_id":"110f289b-359e-4151-96ac-8b81d05e4fc8","remaining_size":"0.125","product_id":"BTC-USD","time":"2016-08-23T17:31:55.504448Z"}"#;
    let op = MakeOrderBookOp(open_packet).unwrap();
    let dut_op = OrderBookOp::Open(UUID::new("110f289b-359e-4151-96ac-8b81d05e4fc8").unwrap(),Side::new("buy").unwrap(), Value::new("582.45").unwrap(), Value::new("0.125").unwrap());
    assert_eq!(op, dut_op);

    let done_packet = r#"{"type":"done","order_type":"limit","side":"sell","sequence":1415928820,"order_id":"f6a70a30-cd05-41f7-b7aa-53e0820d969d","reason":"canceled","product_id":"BTC-USD","time":"2016-08-23T17:31:55.423673Z","price":"589.22","remaining_size":"2.09"}"#;
    let op = MakeOrderBookOp(done_packet).unwrap();
    let dut_op = OrderBookOp::Done(UUID::new("f6a70a30-cd05-41f7-b7aa-53e0820d969d").unwrap(),Side::new("sell").unwrap());
    assert_eq!(op, dut_op);

    let match_packet = r#"{"type":"match","sequence":1415928852,"trade_id":10719547,"maker_order_id":"edfa59ac-3db2-458f-a238-7616ee85437a","taker_order_id":"2d99ff0c-2b5a-4729-be62-7249fb7990e6","side":"sell","size":"0.17","price":"582.49","product_id":"BTC-USD","time":"2016-08-23T17:32:03.042475Z"}"#;
    let op = MakeOrderBookOp(match_packet).unwrap();
    let dut_op = OrderBookOp::Modify(UUID::new("2d99ff0c-2b5a-4729-be62-7249fb7990e6").unwrap(),Side::new("sell").unwrap(), Value::new("0.17").unwrap());
    assert_eq!(op, dut_op);
}



//
//Build structure to hold order
//
use std::collections::BTreeSet;
pub struct OrderBook {
    buy: BTreeSet<Order>,
    sell: BTreeSet<Order>
}
impl OrderBook {
    pub fn new() -> OrderBook {
        OrderBook {
            buy: BTreeSet::new(),
            sell: BTreeSet::new()
        }
    }
    pub fn interact(&mut self, packet: &OrderBookOp ) {
        match packet {
            &OrderBookOp::Ignored => { },
            &OrderBookOp::Error => panic!("Recieved error"),
            &OrderBookOp::Reset => {
                self.buy.clear();
                self.sell.clear();
            },
            &OrderBookOp::Open(ref uuid,Side::Buy,ref p, ref s) => {
                self.buy.insert(Order{ id: uuid.clone(), price: p.clone(), size: s.clone() } );
            },
            &OrderBookOp::Open(ref uuid,Side::Sell,ref p, ref s) => {
                self.sell.insert(Order{ id: uuid.clone(), price: p.clone(), size: s.clone() } );
            },
            &OrderBookOp::Done(ref uuid,Side::Buy) => {
                let temp = Order{ id: uuid.clone(), price: Value::zero(), size: Value::zero() };
                self.buy.remove( &temp );
            },
            &OrderBookOp::Done(ref uuid,Side::Sell) => {
                let temp = Order{ id: uuid.clone(), price: Value::zero(), size: Value::zero() };
                self.sell.remove( &temp );
            },
            &OrderBookOp::Modify(ref uuid, Side::Buy, ref val) => {
                let temp = Order{ id: uuid.clone(), price: Value::zero(), size: Value::zero() };
                //get a copy of the value
                let mut temp_2 = match self.buy.get( &temp ) {
                    Option::None => return,
                    Option::Some(x) => x.clone()
                };
                //decrement it's value it's value
                temp_2.size.data -= val.data;
                //replace the old value
                let _ = self.buy.replace(temp_2);
            },
            &OrderBookOp::Modify(ref uuid, Side::Sell, ref val) => {
                let temp = Order{ id: uuid.clone(), price: Value::zero(), size: Value::zero() };
                //get a copy of the value
                let mut temp_2 = match self.sell.get( &temp ) {
                    Option::None => return,
                    Option::Some(x) => x.clone()
                };
                //decrement it's value it's value
                temp_2.size.data -= val.data;
                //replace the old value
                let _ = self.sell.replace(temp_2);
            }
        }
    }
    pub fn find_min_sell(&self) -> Value {
        self.sell.iter().fold( Value::max(), |x,y| if x < y.price { x } else { y.price } )
    }
    pub fn find_max_sell(&self) -> Value {
        self.sell.iter().fold( Value::zero(), |x,y| if x > y.price { x } else { y.price } )
    }
    pub fn find_min_buy(&self) -> Value {
        self.buy.iter().fold( Value::max(), |x,y| if x < y.price { x } else { y.price } )
    }
    pub fn find_max_buy(&self) -> Value {
        self.buy.iter().fold( Value::zero(), |x,y| if x > y.price { x } else { y.price } )
    }
    pub fn get_buy_vol_at(&self,val: f64) -> f64 {
        let temp = Value::from_f64(val);
        self.buy.iter().filter(|x| x.price == temp).map(|x| x.size.to_f64() ).fold( 0f64, |x,y| x+y )
    }
    pub fn get_sell_vol_at(&self,val: f64) -> f64 {
        let temp = Value::from_f64(val);
        self.sell.iter().filter(|x| x.price == temp).map(|x| x.size.to_f64() ).fold( 0f64, |x,y| x+y )
    }
    //max_buy, min_sell, delta,price
    pub fn spread_value(&self) -> (f64,f64,f64,f64) {
        let max_buy = self.find_max_buy().to_f64();
        let min_sell = self.find_min_sell().to_f64();
        let delta = min_sell - max_buy;
        let price = (min_sell+max_buy)/2f64;
        (max_buy,min_sell,delta,price)
    }
}

//
//This is a thread that'll manage order books all by itself
//
use std::sync::mpsc::{Receiver,Sender,channel};
pub fn order_book_thread( input: Receiver<OrderBookOp> ) {
    //build order book
    let mut book = OrderBook::new();
    for item in input.iter() {
        //execute the order
        book.interact( &item );
        //get information
        let (buy,sell,spread,price) = book.spread_value();
        println!("Buy: {} Sell: {} Delta: {} Price: {}",buy,sell,spread,price);
    }
}

//
//Reads and external path
//
use std::io::prelude::*;
use std::fs::{File, OpenOptions};
use std::io::BufReader;
use std::thread;
pub fn read_parser( path: &str ) {

    //open fifo from websocket server
    let mut fifo: File = match OpenOptions::new().read(true).write(false).open(path) {
        Ok(x) => x,
        Err(e) => panic!("Err: {:?}", e)
    };
    //make the channel
    let (tx,rx) = channel();
    //spawn reporting thread
    thread::spawn(move || {
        order_book_thread(rx);
    });
    //buffer it
    let mut buff = BufReader::new(fifo);
    //read lines
    for line in buff.lines().filter_map(|x| match x {Ok(y) => Some(y), Err(e) => panic!("{:?}",e)} ) {
        match MakeOrderBookOp(&line) {
            Option::None => println!("{}", line),
            Option::Some(OrderBookOp::Ignored) => continue,
            Option::Some(x) => match tx.send(x) {
                Ok(_) => continue,
                Err(e) => panic!("{:?}", e)
            }
        };
    }
}
