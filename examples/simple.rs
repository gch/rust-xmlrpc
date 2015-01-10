extern crate xmlrpc;
extern crate "rustc-serialize" as rustc_serialize;
use rustc_serialize::Encodable;
use rustc_serialize::json;

fn main() {

    println!("This module shows basic functionality of serialization for the XML-RPC protocol.");
    
    println!("\n==== Strings ====");
    let a = "Hello, world!".to_string();
    println!("Before encode: {}", a);
    let b = xmlrpc::encode(&a);
    println!("After encode: {}", b);
    let c: String = xmlrpc::decode(b.as_slice()).unwrap();
    println!("After decode: {}", c);

    println!("\n==== Char ====");
    let a: char = 'a';
    println!("Before encode: {}", a);
    let b = xmlrpc::encode(&a);
    println!("After encode: {}", b);
    let c: char = xmlrpc::decode(b.as_slice()).unwrap();
    println!("After decode: {}", c);

    println!("\n==== Integer ====");
    let a = 18283i32;
    println!("Before encode: {}", a);
    let b = xmlrpc::encode(&a);
    println!("After encode: {}", b);
    let c: i32 = xmlrpc::decode(b.as_slice()).unwrap();
    println!("After decode: {}", c);

    println!("\n==== Floating ====");
    let a = 3.1415926;
    println!("Before encode: {}", a);
    let b = xmlrpc::encode(&a);
    println!("After encode: {}", b);
    let c: f64 = xmlrpc::decode(b.as_slice()).unwrap();
    println!("After decode: {}", c);

    println!("\n==== Booleans ====");
    let a = true;
    println!("Before encode: {}", a);
    let b = xmlrpc::encode(&a);
    println!("After encode: {}", b);
    let c: bool = xmlrpc::decode(b.as_slice()).unwrap();
    println!("After decode: {}", c);

    let a = false;
    println!("Before encode: {}", a);
    let b = xmlrpc::encode(&a);
    println!("After encode: {}", b);
    let c: bool = xmlrpc::decode(b.as_slice()).unwrap();
    println!("After decode: {}", c);

    println!("\n==== Int Vector ====");
    let a = vec![1i32,2,3,4,5,6,7,8];
    println!("Before encode: {:?}", a);
    let b = xmlrpc::encode(&a);
    println!("After encode: {}", b);
    let c: Vec<i32> = xmlrpc::decode(b.as_slice()).unwrap();
    println!("After decode: {:?}", c);

    println!("\n==== Tuple ====");
    let a = ("hello".to_string(), 1.001);
    println!("Before encode: {:?}", a);
    let b = xmlrpc::encode(&a);
    println!("After encode: {}", b);
    let c: (String, f64) = xmlrpc::decode(b.as_slice()).unwrap();
    println!("After decode: {:?}", c);


    println!("\n==== Struct ====");
    #[derive(Show,RustcEncodable,RustcDecodable)]
    struct Person {
        name: String,
        age: i32,
    }
    let a = Person { name: "Dave".to_string(), age: 18 };
    println!("Before encode: {:?}", a);
    let b = xmlrpc::encode(&a);
    println!("After encode: {}", b);
    let c: Person = xmlrpc::decode(b.as_slice()).unwrap();
    println!("After decode: {:?}", c);


    println!("\n==== Vector of Xml values ====");
    let a = vec![xmlrpc::Xml::I32(31),
                 xmlrpc::Xml::Boolean(true),
                 xmlrpc::Xml::String("hello world!".to_string())];
    println!("Before encode: {:?}", a);
    let b = xmlrpc::encode(&a);
    println!("After encode: {}", b);
}
