// Copyright 2014-2015 Galen Clark Haynes
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// Rust XML-RPC library

// Derived from Rust JSON library
// https://github.com/rust-lang/rustc-serialize

use std::collections::{HashMap, BTreeMap};
use std::error::Error as StdError;
use std::mem::{swap, transmute};
use std::num::{Float, Int};
use std::ops::Index;
use std::str::{FromStr};
use std::string;
use std::{char, f64, fmt, io, num, str};
use std;

use rustc_serialize::{Encodable, Decodable};
use rustc_serialize::Encoder as SerializeEncoder;

/// Represents an XML-RPC data value
#[derive(Clone, PartialEq, PartialOrd, Show)]
pub enum Xml {
     I32(i32),
     F64(f64),
     String(string::String),
     Boolean(bool),
     Array(self::Array),
     Object(self::Object),
     Base64(Vec<u8>), // FIXME: added for xml-rpc, not in JSON
     DateTime, // FIXME: need to implement
     Null,
}

pub type Array = Vec<Xml>;
pub type Object = BTreeMap<string::String, Xml>;

/// Shortcut function to encode a `T` into an XML-RPC `String`
pub fn encode<T: Encodable>(object: &T) -> string::String {
    let mut s = String::new();
    {
        let mut encoder = Encoder::new(&mut s);
        let _ = object.encode(&mut encoder);
    }
    s
}

/// Shortcut function to encode a `T` into a XML-RPC `String`
//pub fn encode<'a, T: Encodable<Encoder<'a>, io::IoError>>(object: &T) -> string::String {
//    let buff = Encoder::buffer_encode(object);
//    string::String::from_utf8(buff).unwrap()
//}

pub type EncodeResult = fmt::Result;
//pub type DecodeResult<T> = Result<T, DecoderError>;

fn escape_str(wr: &mut fmt::Writer, v: &str) -> fmt::Result {
    // FIXME: xml encodings
    wr.write_str(v)
}

fn escape_char(writer: &mut fmt::Writer, v: char) -> fmt::Result {
    let mut buf = [0; 4];
    let n = v.encode_utf8(&mut buf).unwrap();
    let buf = unsafe { str::from_utf8_unchecked(&buf[0..n]) };
    escape_str(writer, buf)
}

/// A structure for implementing serialization to XML-RPC.
pub struct Encoder<'a> {
    writer: &'a mut (fmt::Writer+'a),
}

impl<'a> Encoder<'a> {
    /// Creates a new XML-RPC encoder whose output will be written to the writer
    /// specified.
    pub fn new(writer: &'a mut fmt::Writer) -> Encoder<'a> {
        Encoder { writer: writer }
    }
}

impl<'a> SerializeEncoder for Encoder<'a> {
    type Error = fmt::Error;
    fn emit_nil(&mut self) -> EncodeResult { write!(self.writer, "<nil/>") }

    fn emit_usize(&mut self, v: usize) -> EncodeResult { self.emit_i32(v as i32) }
    fn emit_u64(&mut self, v: u64) -> EncodeResult { self.emit_i32(v as i32) }
    fn emit_u32(&mut self, v: u32) -> EncodeResult { self.emit_i32(v as i32) }
    fn emit_u16(&mut self, v: u16) -> EncodeResult { self.emit_i32(v as i32) }
    fn emit_u8(&mut self, v: u8) -> EncodeResult { self.emit_i32(v as i32) }

    fn emit_isize(&mut self, v: isize) -> EncodeResult { self.emit_i32(v as i32) }
    fn emit_i64(&mut self, v: i64) -> EncodeResult { self.emit_i32(v as i32) }
    fn emit_i32(&mut self, v: i32) -> EncodeResult { // XML-RPC only supports 4-byte signed integer
        // FIXME, precondition numbers to check range
        write!(self.writer, "<int>{}</int>", v)
    }
    fn emit_i16(&mut self, v: i16) -> EncodeResult { self.emit_i32(v as i32) }
    fn emit_i8(&mut self, v: i8) -> EncodeResult { self.emit_i32(v as i32) }

    fn emit_bool(&mut self, v: bool) -> EncodeResult {
        write!(self.writer, "<boolean>{}</boolean>", v as u8)
    }

    fn emit_f64(&mut self, v: f64) -> EncodeResult {
        write!(self.writer, "<double>{}</double>", v)
    }
    fn emit_f32(&mut self, v: f32) -> EncodeResult { self.emit_f64(v as f64) }

    fn emit_char(&mut self, v: char) -> EncodeResult {
        try!(write!(self.writer, "<string>"));
        try!(escape_char(self.writer, v));
        write!(self.writer, "</string>")
    }
    fn emit_str(&mut self, v: &str) -> EncodeResult {
        try!(write!(self.writer, "<string>"));
	try!(escape_str(self.writer, v));
        write!(self.writer, "</string>")
    }

    fn emit_enum<F>(&mut self, _name: &str, f: F) -> EncodeResult where
        F: FnOnce(&mut Encoder<'a>) -> EncodeResult,
    {
        f(self)
    }

    fn emit_enum_variant<F>(&mut self,
                            name: &str,
                            _id: usize,
                            cnt: usize,
                            f: F)
                            -> EncodeResult where
        F: FnOnce(&mut Encoder<'a>) -> EncodeResult,
    {
        // enums are encoded as strings or objects
        // Bunny => <string>Bunny</string>
        // Kangaroo(34,"William") =>
        //   <struct>
        //     <member>
        //       <name>variant</name>
        //       <value><string>Kangaroo</string></value>
        //     </member>
 	//     <member>
        //       <name>fields</name>
        //       <value>
        //         <array>
        //           <value><int>34</int></value>
        //           <value><string>William</string></value>
        //         </array>
        //       </value>
        //     </member>
        //   </struct>
        if cnt == 0 {
            self.emit_str(name)
        } else {
            Ok(()) // FIXME
            //IoError<()>
            // FIXME - this is original JSON code below
            //try!(write!(self.writer, "{{\"variant\":"));
            //try!(escape_str(self.writer, name));
            //try!(write!(self.writer, ",\"fields\":["));
            //try!(f(self));
            //write!(self.writer, "]}}")
        }
    }


    fn emit_enum_variant_arg<F>(&mut self, idx: usize, f: F) -> EncodeResult where
        F: FnOnce(&mut Encoder<'a>) -> EncodeResult,
    {
        if idx != 0 {
            try!(write!(self.writer, ","));
        }
        f(self)
    }

    fn emit_enum_struct_variant<F>(&mut self,
                                   name: &str,
                                   id: usize,
                                   cnt: usize,
                                   f: F) -> EncodeResult where
        F: FnOnce(&mut Encoder<'a>) -> EncodeResult,
    {
        self.emit_enum_variant(name, id, cnt, f)
    }

    fn emit_enum_struct_variant_field<F>(&mut self,
                                         _: &str,
                                         idx: usize,
                                         f: F) -> EncodeResult where
        F: FnOnce(&mut Encoder<'a>) -> EncodeResult,
    {
        self.emit_enum_variant_arg(idx, f)
    }

    fn emit_struct<F>(&mut self, _: &str, _: usize, f: F) -> EncodeResult where
        F: FnOnce(&mut Encoder<'a>) -> EncodeResult,
    {
        try!(write!(self.writer, "<struct>"));
        try!(f(self));
        write!(self.writer, "</struct>")
    }

    fn emit_struct_field<F>(&mut self, name: &str, idx: usize, f: F) -> EncodeResult where
        F: FnOnce(&mut Encoder<'a>) -> EncodeResult,
    {
        try!(write!(self.writer, "<member>"));
        try!(write!(self.writer, "<name>{}</name>", name)); // FIXME: encode str?
        try!(write!(self.writer, "<value>"));
        try!(f(self));
        try!(write!(self.writer, "</value>"));
        write!(self.writer, "</member>")
    }

    fn emit_tuple<F>(&mut self, len: usize, f: F) -> EncodeResult where
        F: FnOnce(&mut Encoder<'a>) -> EncodeResult,
    {
        self.emit_seq(len, f)
    }
    fn emit_tuple_arg<F>(&mut self, idx: usize, f: F) -> EncodeResult where
        F: FnOnce(&mut Encoder<'a>) -> EncodeResult,
    {
        self.emit_seq_elt(idx, f)
    }

    fn emit_tuple_struct<F>(&mut self, _name: &str, len: usize, f: F) -> EncodeResult where
        F: FnOnce(&mut Encoder<'a>) -> EncodeResult,
    {
        self.emit_seq(len, f)
    }
    fn emit_tuple_struct_arg<F>(&mut self, idx: usize, f: F) -> EncodeResult where
        F: FnOnce(&mut Encoder<'a>) -> EncodeResult,
    {
        self.emit_seq_elt(idx, f)
    }

    fn emit_option<F>(&mut self, f: F) -> EncodeResult where
        F: FnOnce(&mut Encoder<'a>) -> EncodeResult,
    {
        f(self)
    }
    fn emit_option_none(&mut self) -> EncodeResult { self.emit_nil() }
    fn emit_option_some<F>(&mut self, f: F) -> EncodeResult where
        F: FnOnce(&mut Encoder<'a>) -> EncodeResult,
    {
        f(self)
    }

    fn emit_seq<F>(&mut self, _len: usize, f: F) -> EncodeResult where
        F: FnOnce(&mut Encoder<'a>) -> EncodeResult,
    {
        try!(write!(self.writer, "<array><data>"));
        try!(f(self));
        write!(self.writer, "</data></array>")
    }

    fn emit_seq_elt<F>(&mut self, idx: usize, f: F) -> EncodeResult where
        F: FnOnce(&mut Encoder<'a>) -> EncodeResult,
    {
        try!(write!(self.writer, "<value>"));
        try!(f(self));
        write!(self.writer, "</value>")
    }

    fn emit_map<F>(&mut self, _len: usize, f: F) -> EncodeResult where
        F: FnOnce(&mut Encoder<'a>) -> EncodeResult,
    {
        Ok(())
        // FIXME: this is JSON source
        //try!(write!(self.writer, "{{"));
        //try!(f(self));
        //write!(self.writer, "}}")
    }

    //fn emit_map_elt_key<F>(&mut self, idx: usize, mut f: F) -> EncodeResult where
    // FIXME: implement
    fn emit_map_elt_key<F>(&mut self, idx: usize, f: F) -> EncodeResult where
        F: FnMut(&mut Encoder<'a>) -> EncodeResult,
    {
        //if idx != 0 { try!(write!(self.writer, ",")) }
        //// ref #12967, make sure to wrap a key in double quotes,
        //// in the event that its of a type that omits them (eg numbers)
        //let mut buf = Vec::new();
        // // FIXME(14302) remove the transmute and unsafe block.
        //unsafe {
        //    let mut check_encoder = Encoder::new(&mut buf);
        //    try!(f(transmute(&mut check_encoder)));
        //}
        //let out = str::from_utf8(buf[]).unwrap();
        //let needs_wrapping = out.char_at(0) != '"' && out.char_at_reverse(out.len()) != '"';
        //if needs_wrapping { try!(write!(self.writer, "\"")); }
        //try!(f(self));
        //if needs_wrapping { try!(write!(self.writer, "\"")); }
        Ok(())
    }

    fn emit_map_elt_val<F>(&mut self, _idx: usize, f: F) -> EncodeResult where
        F: FnOnce(&mut Encoder<'a>) -> EncodeResult,
    {
        Ok(())
        //try!(write!(self.writer, ":"));
        //f(self)
    }
}

impl Encodable for Xml {
    fn encode<S: SerializeEncoder>(&self, e: &mut S) -> Result<(), S::Error> {
        match *self {
            Xml::I32(v) => v.encode(e),
            Xml::F64(v) => v.encode(e),
            Xml::String(ref v) => v.encode(e),
            Xml::Boolean(v) => v.encode(e),
            Xml::Array(ref v) => v.encode(e),
            Xml::Object(ref v) => v.encode(e), // FIXME: had to add hardcoded
                                               // impl for BTreeMap
            Xml::Null => e.emit_nil(),
            _ => Ok(()), // FIXME: add other types
        }
    }
}

impl Xml {
    /// If the XML value is an Object, returns the value associated with the provided key.
    /// Otherwise, returns None.
    pub fn find<'a>(&'a self, key: &str) -> Option<&'a Xml>{
        match self {
            &Xml::Object(ref map) => map.get(key),
            _ => None
        }
    }

    /// Attempts to get a nested XML Object for each key in `keys`.
    /// If any key is found not to exist, find_path will return None.
    /// Otherwise, it will return the Xml value associated with the final key.
    pub fn find_path<'a>(&'a self, keys: &[&str]) -> Option<&'a Xml>{
        let mut target = self;
        for key in keys.iter() {
            match target.find(*key) {
                Some(t) => { target = t; },
                None => return None
            }
        }
        Some(target)
    }

    /// If the XML value is an Object, performs a depth-first search until
    /// a value associated with the provided key is found. If no value is found
    /// or the XML value is not an Object, returns None.
    pub fn search<'a>(&'a self, key: &str) -> Option<&'a Xml> {
        match self {
            &Xml::Object(ref map) => {
                match map.get(key) {
                    Some(xml_value) => Some(xml_value),
                    None => {
                        for (_, v) in map.iter() {
                            match v.search(key) {
                                x if x.is_some() => return x,
                                _ => ()
                            }
                        }
                        None
                    }
                }
            },
            _ => None
        }
    }

    /// Returns true if the XML value is an Object. Returns false otherwise.
    pub fn is_object<'a>(&'a self) -> bool {
        self.as_object().is_some()
    }

    /// If the XML value is an Object, returns the associated BTreeMap.
    /// Returns None otherwise.
    pub fn as_object<'a>(&'a self) -> Option<&'a Object> {
        match self {
            &Xml::Object(ref map) => Some(map),
            _ => None
        }
    }

    /// Returns true if the XML value is an Array. Returns false otherwise.
    pub fn is_array<'a>(&'a self) -> bool {
        self.as_array().is_some()
    }

    /// If the XML value is an Array, returns the associated vector.
    /// Returns None otherwise.
    pub fn as_array<'a>(&'a self) -> Option<&'a Array> {
        match self {
            &Xml::Array(ref array) => Some(&*array),
            _ => None
        }
    }

    /// Returns true if the XML value is a String. Returns false otherwise.
    pub fn is_string<'a>(&'a self) -> bool {
        self.as_string().is_some()
    }

    /// If the Xml value is a String, returns the associated str.
    /// Returns None otherwise.
    pub fn as_string<'a>(&'a self) -> Option<&'a str> {
        match *self {
            Xml::String(ref s) => Some(s.as_slice()),
            _ => None
        }
    }

    /// Returns true if the XML value is a Number. Returns false otherwise.
    pub fn is_number(&self) -> bool {
        match *self {
            Xml::I32(_) | Xml::F64(_) => true,
            _ => false,
        }
    }

    /// Returns true if the XML value is a i32. Returns false otherwise.
    pub fn is_i32(&self) -> bool {
        match *self {
            Xml::I32(_) => true,
            _ => false,
        }
    }

    /// Returns true if the XML value is a f64. Returns false otherwise.
    pub fn is_f64(&self) -> bool {
        match *self {
            Xml::F64(_) => true,
            _ => false,
        }
    }

    /// If the XML value is a number, return or cast it to a i64.
    /// Returns None otherwise.
    pub fn as_i32(&self) -> Option<i32> {
        match *self {
            Xml::I32(n) => Some(n),
            _ => None
        }
    }

    /// If the XML value is a number, return or cast it to a f64.
    /// Returns None otherwise.
    pub fn as_f64(&self) -> Option<f64> {
        match *self {
            Xml::I32(n) => num::cast(n),
            Xml::F64(n) => Some(n),
            _ => None
        }
    }

    /// Returns true if the Xml value is a Boolean. Returns false otherwise.
    pub fn is_boolean(&self) -> bool {
        self.as_boolean().is_some()
    }

    /// If the Xml value is a Boolean, returns the associated bool.
    /// Returns None otherwise.
    pub fn as_boolean(&self) -> Option<bool> {
        match self {
            &Xml::Boolean(b) => Some(b),
            _ => None
        }
    }

    /// Returns true if the XML value is a Null. Returns false otherwise.
    pub fn is_null(&self) -> bool {
        self.as_null().is_some()
    }

    /// If the XML value is a Null, returns ().
    /// Returns None otherwise.
    pub fn as_null(&self) -> Option<()> {
        match self {
            &Xml::Null => Some(()),
            _ => None
        }
    }
}

impl<'a> Index<&'a str>  for Xml {
    type Output = Xml;

    fn index(&self, idx: & &str) -> &Xml {
        self.find(*idx).unwrap()
    }
}

impl Index<usize> for Xml {
    type Output = Xml;

    fn index<'a>(&'a self, idx: &usize) -> &'a Xml {
        match self {
            &Xml::Array(ref v) => v.index(idx),
            _ => panic!("can only index XML with usize if it is an array")
        }
    }
}

/// A trait for converting values to XML
pub trait ToXml {
    /// Converts the value of `self` to an instance of XML
    fn to_xml(&self) -> Xml;
}

macro_rules! to_xml_impl_i32 {
    ($($t:ty), +) => (
        $(impl ToXml for $t {
            fn to_xml(&self) -> Xml { Xml::I32(*self as i32) }
        })+
    )
}

to_xml_impl_i32! { isize, i8, i16, i32, i64 }
to_xml_impl_i32! { usize, u8, u16, u32, u64 }

impl ToXml for Xml {
    fn to_xml(&self) -> Xml { self.clone() }
}

impl ToXml for f32 {
    fn to_xml(&self) -> Xml { (*self as f64).to_xml() }
}

impl ToXml for f64 {
    fn to_xml(&self) -> Xml {
        Xml::F64(*self)
        /* // FIXME: look up XML-RPC float behavior
        use std::num::FpCategory::{Nan, Infinite};

        match self.classify() {
            Nan | Infinite => Xml::Null,
            _                  => Xml::F64(*self)
        }
        */
    }
}

impl ToXml for () {
    fn to_xml(&self) -> Xml { Xml::Null }
}

impl ToXml for bool {
    fn to_xml(&self) -> Xml { Xml::Boolean(*self) }
}

impl ToXml for str {
    fn to_xml(&self) -> Xml { Xml::String(self.to_string()) }
}

impl ToXml for string::String {
    fn to_xml(&self) -> Xml { Xml::String((*self).clone()) }
}

macro_rules! tuple_impl {
    // use variables to indicate the arity of the tuple
    ($($tyvar:ident),* ) => {
        // the trailing commas are for the 1 tuple
        impl<
            $( $tyvar : ToXml ),*
            > ToXml for ( $( $tyvar ),* , ) {

            #[inline]
            #[allow(non_snake_case)]
            fn to_xml(&self) -> Xml {
                match *self {
                    ($(ref $tyvar),*,) => Xml::Array(vec![$($tyvar.to_xml()),*])
                }
            }
        }
    }
}

tuple_impl!{A}
tuple_impl!{A, B}
tuple_impl!{A, B, C}
tuple_impl!{A, B, C, D}
tuple_impl!{A, B, C, D, E}
tuple_impl!{A, B, C, D, E, F}
tuple_impl!{A, B, C, D, E, F, G}
tuple_impl!{A, B, C, D, E, F, G, H}
tuple_impl!{A, B, C, D, E, F, G, H, I}
tuple_impl!{A, B, C, D, E, F, G, H, I, J}
tuple_impl!{A, B, C, D, E, F, G, H, I, J, K}
tuple_impl!{A, B, C, D, E, F, G, H, I, J, K, L}

impl<A: ToXml> ToXml for [A] {
    fn to_xml(&self) -> Xml { Xml::Array(self.iter().map(|elt| elt.to_xml()).collect()) }
}

impl<A: ToXml> ToXml for Vec<A> {
    fn to_xml(&self) -> Xml { Xml::Array(self.iter().map(|elt| elt.to_xml()).collect()) }
}

impl<A: ToXml> ToXml for BTreeMap<string::String, A> {
    fn to_xml(&self) -> Xml {
        let mut d = BTreeMap::new();
        for (key, value) in self.iter() {
            d.insert((*key).clone(), value.to_xml());
        }
        Xml::Object(d)
    }
}

impl<A: ToXml> ToXml for HashMap<string::String, A> {
    fn to_xml(&self) -> Xml {
        let mut d = BTreeMap::new();
        for (key, value) in self.iter() {
            d.insert((*key).clone(), value.to_xml());
        }
        Xml::Object(d)
    }
}

impl<A:ToXml> ToXml for Option<A> {
    fn to_xml(&self) -> Xml {
        match *self {
            None => Xml::Null,
            Some(ref value) => value.to_xml()
        }
    }
}

#[cfg(test)]
mod tests {

}
