
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
}
unsafe impl Send for Value {

}
#[test]
fn test_value() {
    let dut_str = "5872.75856521";
    let dut_val = Value::new(dut_str).unwrap();
    assert_eq!(dut_val.data, 58727585652100u64);

    let dut_str = "0";
    let dut_val = Value::new(dut_str).unwrap();
    assert_eq!(dut_val.data, 0);

    let dut_str = "null";
    let dut_val = Value::new(dut_str).unwrap();
    assert_eq!(dut_val.data, 0);

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
    id: UUID,
    price: Value,
    size: Value
}
unsafe impl Send for Order {
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
