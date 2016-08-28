
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

}

//
//Represents a value from
//99,999,999.9999999999
//          10000000000
//
#[derive(Clone,Copy,PartialEq,Eq,PartialOrd,Ord)]
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
#[derive(Clone,Copy,PartialEq,Eq,PartialOrd,Ord)]
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


/*
//
//This is a high level representation of a transaction
//
#[derive(Clone,Debug)]
pub enum OrderBookOp {
    Rece
}

#[test]
fn test_read_packet() {
    let dut_str = r#""#;


}
*/
