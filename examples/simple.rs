extern crate xmlrpc;
extern crate serialize;
use serialize::Encodable;
use serialize::json;

fn main() {

    println!("XML of string: {}", xmlrpc::encode(&"hello world!".to_string()));
    println!("XML of char: {}", xmlrpc::encode(&'c'));
    println!("XML of int: {}", xmlrpc::encode(&9248i));
    println!("XML of float: {}", xmlrpc::encode(&19824.28824f32));
    println!("XML of bool (true): {}", xmlrpc::encode(&true));
    println!("XML of bool (false): {}", xmlrpc::encode(&false));

    #[deriving(Encodable)]
    struct MyStruct {
        value: String
    }
    let m: MyStruct = MyStruct { value: "foobar".to_string() };
    println!("XML of struct: {}", xmlrpc::encode(&m));

    let v = vec![1i,2i,3i,4i];
    println!("XML of int vector: {}", xmlrpc::encode(&v));

    println!("XML of string Xml object: {}", xmlrpc::encode(&xmlrpc::Xml::I64(32i64)));
    println!("XML of string Xml object: {}", xmlrpc::encode(&xmlrpc::Xml::Boolean(false)));

    #[deriving(Encodable)]
    struct City {
        name: &'static str,
        // Latitude
        lat: f32,
        // Longitude
        lon: f32,
    }

    for city in [
        City { name: "São Paulo", lat: -23.55,     lon: -46.633333 },
        City { name: "Lima",      lat: -12.043333, lon: -77.028333 },
        City { name: "Santiago",  lat: -33.45,     lon: -70.666667 },
    ].iter() {
        // `encode` encodes an `Encodable` implementor into a `String`
        println!("JSON: {}", json::encode(city));
        println!("XML: {}", xmlrpc::encode(city));
    }
}
