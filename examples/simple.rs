extern crate xmlrpc;
extern crate serialize;
use serialize::Encodable;
use serialize::json;

fn main() {

    println!("XML of string: {}", xmlrpc::encode(&"hello world!".to_string()));
    println!("XML of char: {}", xmlrpc::encode(&'c'));
    println!("XML of int: {}", xmlrpc::encode(&9248is));
    println!("XML of float: {}", xmlrpc::encode(&19824.28824f32));
    println!("XML of bool (true): {}", xmlrpc::encode(&true));
    println!("XML of bool (false): {}", xmlrpc::encode(&false));

    #[deriving(Encodable)]
    struct MyStruct {
        value: String
    }
    let m: MyStruct = MyStruct { value: "foobar".to_string() };
    //println!("XML of struct: {}", xmlrpc::encode(&m)); // FIXME: fix object encoding

    let v = vec![1i32,2,3,4];
    println!("XML of int vector: {}", xmlrpc::encode(&v));

    println!("XML of string Xml object: {}", xmlrpc::encode(&xmlrpc::Xml::I32(32)));
    println!("XML of string Xml object: {}", xmlrpc::encode(&xmlrpc::Xml::Boolean(false)));

    let v = vec![xmlrpc::Xml::I32(31),
                 xmlrpc::Xml::Boolean(true),
                 xmlrpc::Xml::String("hello world!".to_string())];
    println!("XML of xml vector: {}", xmlrpc::encode(&v));
    println!("JSON of same xml vector: {}", json::encode(&v));

    // grab a string value out of an xml value
    println!("Is index 0 a string? {}", v[1].is_string());
    println!("Is index 1 a string? {}", v[1].is_string());
    println!("Is index 2 a string? {}", v[2].is_string());
    println!("Index 2 as a string: {}", v[2].as_string().unwrap());

    // make an Xml object of an arbitrary struct
    // FIXME: is there a way to automatically translate this into BTreeMap?
    //#[deriving(Encodable)]
    //struct Person {
    //    name: &'static str,
    //    age: i32
    //}
    //let p = Person { name: "Clark", age: 35 };
    //println!("XML of person: {}", xmlrpc::encode(&xmlrpc::Xml::Object(p)));

    /* // FIXME: add back in after we re-enable object encoding
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
    */
}
