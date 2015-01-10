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

use self::ErrorCode::*;
use self::ParserError::*;
use self::DecoderError::*;

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
use rustc_serialize::Decoder as SerializeDecoder;

use xml;
use xml::EventReader;
use xml::reader::events;

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

pub struct AsXml<'a, T: 'a> { inner: &'a T }

/// The errors that can arise while parsing an XML stream.
#[derive(Clone, Copy, PartialEq)]
pub enum ErrorCode {
    InvalidSyntax,
    EOFWhileParsingObject,
    EOFWhileParsingArray,
    EOFWhileParsingValue,
    EOFWhileParsingString,
}

#[derive(Clone, Copy, PartialEq, Show)]
pub enum ParserError {
    /// msg, line, col
    SyntaxError(ErrorCode, usize, usize),
    IoError(io::IoErrorKind, &'static str),
}

// Builder and Parser have the same errors.
pub type BuilderError = ParserError;

#[derive(Clone, PartialEq, Show)]
pub enum DecoderError {
    ParseError(ParserError),
    ExpectedError(string::String, string::String),
    MissingFieldError(string::String),
    UnknownVariantError(string::String),
    ApplicationError(string::String)
}

/// Returns a readable error string for a given error code.
pub fn error_str(error: ErrorCode) -> &'static str {
    match error {
        InvalidSyntax => "invalid syntax",
        EOFWhileParsingObject => "EOF While parsing object",
        EOFWhileParsingArray => "EOF While parsing array",
        EOFWhileParsingValue => "EOF While parsing value",
        EOFWhileParsingString => "EOF While parsing string",
    }
}

/*
/// Shortcut function to decode a XML `&str` into an object
pub fn decode<T: Decodable>(s: &str) -> DecodeResult<T> {
    let xml = match Xml::from_str(s) {
        Ok(x) => x,
        Err(e) => return Err(ParseError(e))
    };

    let mut decoder = Decoder::new(xml);
    Decodable::decode(&mut decoder)
}
*/
/// Shortcut function to encode a `T` into an XML `String`
pub fn encode<T: Encodable>(object: &T) -> string::String {
    let mut s = String::new();
    {
        let mut encoder = Encoder::new(&mut s);
        let _ = object.encode(&mut encoder);
    }
    s
}

impl fmt::Show for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        error_str(*self).fmt(f)
    }
}

fn io_error_to_error(io: io::IoError) -> ParserError {
    ParserError::IoError(io.kind, io.desc)
}

impl StdError for DecoderError {
    fn description(&self) -> &str { "decoder error" }
    fn detail(&self) -> Option<std::string::String> { Some(format!("{:?}", self)) }
    fn cause(&self) -> Option<&StdError> {
        match *self {
            DecoderError::ParseError(ref e) => Some(e as &StdError),
            _ => None,
        }
    }
}

impl StdError for ParserError {
    fn description(&self) -> &str { "failed to parse xml" }
    fn detail(&self) -> Option<std::string::String> { Some(format!("{:?}", self)) }
}

pub type EncodeResult = fmt::Result;
pub type DecodeResult<T> = Result<T, DecoderError>;

fn escape_str(wr: &mut fmt::Writer, v: &str) -> fmt::Result {
    wr.write_str(xml::escape::escape_str(v).as_slice())
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

/// Create an `AsXml` wrapper which can be used to print a value as XML
/// on-the-fly via `write!`
pub fn as_xml<T: Encodable>(t: &T) -> AsXml<T> {
    AsXml { inner: t }
}


impl Xml {

    pub fn from_str(s: &str) -> Result<Self, BuilderError> {
        //let mut builder = Builder::new(s.chars());
        //builder.build()
        let rdr = io::MemReader::new(String::from_str(s).into_bytes());
        let brdr = io::BufferedReader::new(rdr);
        let mut builder = Builder::new(brdr);
        builder.build()
    }

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

/// The output of the streaming parser.
#[derive(PartialEq, Clone, Show)]
pub enum XmlEvent {
    ObjectStart, // <struct>
    ObjectEnd, // </struct>
    MemberStart, // <member>
    MemberEnd, // </member>
    NameStart, // <name>
    NameValue(string::String),
    NameEnd, // </name>
    ValueStart, // <value>
    ValueEnd, // </value>
    ArrayStart, // <array>
    ArrayEnd, // </array>
    DataStart, // <data>
    DataEnd, // </data>
    BooleanStart, // <boolean>
    BooleanValue(bool),
    BooleanEnd, // </boolean>
    I32Start, // <int> or <i4>
    I32Value(i32),
    I32End, // </int> or </i4>
    F64Start, // <double>
    F64Value(f64),
    F64End, // </double>
    StringStart, // <string>
    StringValue(string::String),
    StringEnd, // </string>
    NullStart, // <nil/>
    NullEnd, // <nil/>
    // FIXME: datetime
    // FIXME: Base64
    Error(ParserError) // FIXME: add error types
}

struct Builder<B: Buffer> {
    parser: EventReader<B>,
    token: Option<XmlEvent>,
}

impl<B: Buffer> Builder<B> {
    /// Create an XML Builder.
    pub fn new(src: B) -> Builder<B> {
        Builder { parser: EventReader::new(src), token: None, }
    }


    pub fn build(&mut self) -> Result<Xml, BuilderError> {
        self.bump();
        let result = self.build_value();
        self.bump();
        match self.token {
            None => {}
            Some(XmlEvent::Error(e)) => { return Err(e); }
            ref tok => { panic!("unexpected token {:?}", tok.clone()); }
        }
        result
        /* // previous, simpler version
        match self.build_value() {
            Ok(v) => Ok(v),
            Err(e) => Err(DecoderError::MissingFieldError("foobar".to_string())), // FIXME
        }
        */
    }

    fn bump(&mut self) {
        let mut n = self.parser.next();
        loop {
            match n {
                // FIXME: terser version
                events::XmlEvent::StartDocument{version: _, encoding: _, standalone: _} => (),
                _ => break,
            }
            n = self.parser.next();
        }
        self.token = match n {
            events::XmlEvent::StartElement { name, attributes: _, namespace: _ } => {
                self.parse_tag_start(name.local_name.as_slice())
            }
            events::XmlEvent::EndElement { name } => {
                self.parse_tag_end(name.local_name.as_slice())
            }
            events::XmlEvent::Characters(s) => {
                self.parse_tag_characters(s.as_slice(), &self.token)
            }
            events::XmlEvent::EndDocument => {
                None
            }
            _ => None,
        }
    }

    pub fn build_value(&mut self) -> Result<Xml, BuilderError> {
        match self.token {
            // all values must begin with opening tag
            Some(XmlEvent::ObjectStart) => self.build_object(),
            Some(XmlEvent::ArrayStart) => self.build_array(),
            Some(XmlEvent::NullStart) => self.build_nil(),
            Some(XmlEvent::I32Start) => self.build_i32(),
            Some(XmlEvent::F64Start) => self.build_f64(),
            Some(XmlEvent::BooleanStart) => self.build_boolean(),
            Some(XmlEvent::StringStart) => self.build_string(),
            // error otherwise
            Some(XmlEvent::ObjectEnd) => Err(SyntaxError(InvalidSyntax, 0, 0)),
            Some(XmlEvent::ArrayEnd) => Err(SyntaxError(InvalidSyntax, 0, 0)),
            Some(XmlEvent::NullEnd) => Err(SyntaxError(InvalidSyntax, 0, 0)),
            Some(XmlEvent::I32End) => Err(SyntaxError(InvalidSyntax, 0, 0)),
            Some(XmlEvent::F64End) => Err(SyntaxError(InvalidSyntax, 0, 0)),
            Some(XmlEvent::BooleanEnd) => Err(SyntaxError(InvalidSyntax, 0, 0)),
            Some(XmlEvent::StringEnd) => Err(SyntaxError(InvalidSyntax, 0, 0)),
            Some(XmlEvent::NameStart) => Err(SyntaxError(InvalidSyntax, 0, 0)),
            Some(XmlEvent::NameEnd) => Err(SyntaxError(InvalidSyntax, 0, 0)),
            Some(XmlEvent::MemberStart) => Err(SyntaxError(InvalidSyntax, 0, 0)),
            Some(XmlEvent::MemberEnd) => Err(SyntaxError(InvalidSyntax, 0, 0)),
            Some(XmlEvent::DataStart) => Err(SyntaxError(InvalidSyntax, 0, 0)),
            Some(XmlEvent::DataEnd) => Err(SyntaxError(InvalidSyntax, 0, 0)),
            Some(XmlEvent::ValueStart) => Err(SyntaxError(InvalidSyntax, 0, 0)),
            Some(XmlEvent::ValueEnd) => Err(SyntaxError(InvalidSyntax, 0, 0)),
            Some(XmlEvent::I32Value(_)) => Err(SyntaxError(InvalidSyntax, 0, 0)),
            Some(XmlEvent::F64Value(_)) => Err(SyntaxError(InvalidSyntax, 0, 0)),
            Some(XmlEvent::BooleanValue(_)) => Err(SyntaxError(InvalidSyntax, 0, 0)),
            Some(XmlEvent::StringValue(_)) => Err(SyntaxError(InvalidSyntax, 0, 0)),
            Some(XmlEvent::NameValue(_)) => Err(SyntaxError(InvalidSyntax, 0, 0)),
            Some(XmlEvent::Error(e)) => Err(e),
            None => Err(SyntaxError(EOFWhileParsingValue,0,0)),
        }
    }

    fn build_object(&mut self) -> Result<Xml, BuilderError> {
        self.bump();
        let mut values = BTreeMap::new();
        loop {
            match self.token {
                Some(XmlEvent::ObjectEnd) => {
                    return Ok(Xml::Object(values));
                }
                _ => {}
            }
            // FIXME: use error codes appropriate for the cause
            // looking for <member>
            if self.token != Some(XmlEvent::MemberStart) {
                return Err(SyntaxError(InvalidSyntax,0,0));
            }
            self.bump(); // looking for <name>
            if self.token != Some(XmlEvent::NameStart) {
                return Err(SyntaxError(InvalidSyntax,0,0));
            }
            self.bump(); // looking for string value inside name
            let key = match self.token {
                Some(XmlEvent::NameValue(ref s)) => s.to_string(),
                _ => { return Err(SyntaxError(InvalidSyntax,0,0)); }
            };
            self.bump(); // looking for </name>
            if self.token != Some(XmlEvent::NameEnd) {
                return Err(SyntaxError(InvalidSyntax,0,0));
            }
            self.bump(); // looking for <value>
            if self.token != Some(XmlEvent::ValueStart) {
                return Err(SyntaxError(InvalidSyntax,0,0));
            }
            self.bump(); // parse whatever value is inside
            match self.build_value() {
                Ok(value) => { values.insert(key, value); }
                Err(e) => { return Err(e); }
            }
            self.bump(); // looking for </value>
            if self.token != Some(XmlEvent::ValueEnd) {
                return Err(SyntaxError(InvalidSyntax,0,0));
            }
            self.bump(); // looking for </member>
            if self.token != Some(XmlEvent::MemberEnd) {
                return Err(SyntaxError(InvalidSyntax,0,0));
            }
            self.bump();
        }
    }

    fn build_array(&mut self) -> Result<Xml, BuilderError> {
        self.bump();
        let mut values = Vec::new();
        loop {
            if self.token == Some(XmlEvent::ArrayEnd) {
                return Ok(Xml::Array(values.into_iter().collect()));
            }
            if self.token == Some(XmlEvent::ValueStart) {
                self.bump();
                match self.build_value() {
                    Ok(v) => values.push(v),
                    Err(e) => { return Err(e) }
                }
                self.bump();
                match self.token {
                    Some(XmlEvent::ValueEnd) => (),
                    _ => { return Err(SyntaxError(InvalidSyntax,0,0)); }
                }
            }
            self.bump();
        }
    }

    fn build_nil(&mut self) -> Result<Xml, BuilderError> {
        self.bump();
        match self.token {
            Some(XmlEvent::NullEnd) => Ok(Xml::Null),
            _ => Err(SyntaxError(InvalidSyntax,0,0)),
        }
    }

    fn build_boolean(&mut self) -> Result<Xml, BuilderError> {
        self.bump();
        let val = match self.token {
            Some(XmlEvent::BooleanValue(b)) => Ok(Xml::Boolean(b)), // FIXME
            _ => Err(SyntaxError(InvalidSyntax,0,0)),
        };
        self.bump();
        match self.token {
            Some(XmlEvent::BooleanEnd) => val,
            _ => Err(SyntaxError(InvalidSyntax,0,0)),
        }
    }

    fn build_i32(&mut self) -> Result<Xml, BuilderError> {
        self.bump();
        let val = match self.token {
            Some(XmlEvent::I32Value(v)) => Ok(Xml::I32(v)),
            _ => Err(SyntaxError(InvalidSyntax,0,0)),
        };
        self.bump();
        match self.token {
            Some(XmlEvent::I32End) => val,
            _ => Err(SyntaxError(InvalidSyntax,0,0)),
        }
    }

    fn build_f64(&mut self) -> Result<Xml, BuilderError> {
        self.bump();
        let val = match self.token {
            Some(XmlEvent::F64Value(v)) => Ok(Xml::F64(v)),
            _ => Err(SyntaxError(InvalidSyntax,0,0)),
        };
        self.bump();
        match self.token {
            Some(XmlEvent::F64End) => val,
            _ => Err(SyntaxError(InvalidSyntax,0,0)),
        }
    }

    fn build_string(&mut self) -> Result<Xml, BuilderError> {
        self.bump();
        let val = match self.token {
            Some(XmlEvent::StringValue(ref s)) => Ok(Xml::String(s.to_string())),
            _ => Err(SyntaxError(InvalidSyntax,0,0)),
        };
        self.bump();
        match self.token {
            Some(XmlEvent::StringEnd) => val,
            _ => Err(SyntaxError(InvalidSyntax,0,0)),
        }
    }

    fn parse_bool_value(&self, s: &str) -> Option<XmlEvent> {
        match s {
            "0" => Some(XmlEvent::BooleanValue(false)),
            "1" => Some(XmlEvent::BooleanValue(true)),
            _ => None
        }
    }

    fn parse_i32_value(&self, s: &str) -> Option<XmlEvent> {
        match s.parse::<i32>() {
            Some(n) => Some(XmlEvent::I32Value(n)),
            None => None
        }
    }
    fn parse_f64_value(&self, s: &str) -> Option<XmlEvent> {
        match s.parse::<f64>() {
            Some(n) => Some(XmlEvent::F64Value(n)),
            None => None
        }
    }
    fn parse_string_value(&self, s: &str) -> Option<XmlEvent> {
        Some(XmlEvent::StringValue(s.to_string()))
    }
    fn parse_name_value(&self, s: &str) -> Option<XmlEvent> {
        Some(XmlEvent::NameValue(s.to_string()))
    }
    fn parse_tag_start(&self, name: &str) -> Option<XmlEvent> {
        return match name {
            "struct" => Some(XmlEvent::ObjectStart),
            "member" => Some(XmlEvent::MemberStart),
            "name" => Some(XmlEvent::NameStart),
            "value" => Some(XmlEvent::ValueStart),
            "array" => Some(XmlEvent::ArrayStart),
            "data" => Some(XmlEvent::DataStart),
            "boolean" => Some(XmlEvent::BooleanStart),
            "int" => Some(XmlEvent::I32Start),
            "double" => Some(XmlEvent::F64Start),
            "string" => Some(XmlEvent::StringStart),
            "nil" => Some(XmlEvent::NullStart),
            _ => None,
        }
    }

    fn parse_tag_end(&self, name: &str) -> Option<XmlEvent> {
        return match name {
            "struct" => Some(XmlEvent::ObjectEnd),
            "member" => Some(XmlEvent::MemberEnd),
            "name" => Some(XmlEvent::NameEnd),
            "value" => Some(XmlEvent::ValueEnd),
            "array" => Some(XmlEvent::ArrayEnd),
            "data" => Some(XmlEvent::DataEnd),
            "boolean" => Some(XmlEvent::BooleanEnd),
            "int" => Some(XmlEvent::I32End),
            "double" => Some(XmlEvent::F64End),
            "string" => Some(XmlEvent::StringEnd),
            "nil" => Some(XmlEvent::NullEnd),
            _ => None,
        }
    }

    fn parse_tag_characters(&self, s: &str, token: &Option<XmlEvent>) -> Option<XmlEvent> {
        match token {
            &Some(XmlEvent::BooleanStart) => self.parse_bool_value(s),
            &Some(XmlEvent::I32Start) => self.parse_i32_value(s),
            &Some(XmlEvent::F64Start) => self.parse_f64_value(s),
            &Some(XmlEvent::StringStart) => self.parse_string_value(s),
            &Some(XmlEvent::NameStart) => self.parse_name_value(s),
            _ => None,
        }
    }
}

/// A structure to decode JSON to values in rust.
pub struct Decoder {
    stack: Vec<Xml>,
}

impl Decoder {
    /// Creates a new decoder instance for decoding the specified XML value.
    pub fn new(xml: Xml) -> Decoder {
        Decoder { stack: vec![xml] }
    }
}

impl Decoder {
    fn pop(&mut self) -> Xml {
        self.stack.pop().unwrap()
    }
}

macro_rules! expect {
    ($e:expr, Null) => ({
        match $e {
            Xml::Null => Ok(()),
            other => Err(ExpectedError("Null".to_string(),
                                       format!("{}", other)))
        }
    });
    ($e:expr, $t:ident) => ({
        match $e {
            Xml::$t(v) => Ok(v),
            other => {
                Err(ExpectedError(stringify!($t).to_string(),
                                  format!("{}", other)))
            }
        }
    })
}

macro_rules! read_primitive {
    ($name:ident, $ty:ty) => {
        fn $name(&mut self) -> DecodeResult<$ty> {
            match self.pop() {
                Xml::I32(f) => match num::cast(f) {
                    Some(f) => Ok(f),
                    None => Err(ExpectedError("Number".to_string(), format!("{}", f))),
                },
                Xml::F64(f) => Err(ExpectedError("Integer".to_string(), format!("{}", f))),
                Xml::String(s) => match s.parse() {
                    Some(f) => Ok(f),
                    None => Err(ExpectedError("Number".to_string(), s)),
                },
                value => Err(ExpectedError("Number".to_string(), format!("{}", value))),
            }
        }
    }
}

impl SerializeDecoder for Decoder {
    type Error = DecoderError;

    fn read_nil(&mut self) -> DecodeResult<()> {
        expect!(self.pop(), Null)
    }

    read_primitive! { read_usize, usize }
    read_primitive! { read_u8, u8 }
    read_primitive! { read_u16, u16 }
    read_primitive! { read_u32, u32 }
    read_primitive! { read_u64, u64 }
    read_primitive! { read_isize, isize }
    read_primitive! { read_i8, i8 }
    read_primitive! { read_i16, i16 }
    read_primitive! { read_i32, i32 }
    read_primitive! { read_i64, i64 }

    fn read_f32(&mut self) -> DecodeResult<f32> { self.read_f64().map(|x| x as f32) }

    fn read_f64(&mut self) -> DecodeResult<f64> {
        match self.pop() {
            Xml::I32(f) => Ok(f as f64),
            Xml::F64(f) => Ok(f),
            Xml::String(s) => { // FIXME: does this exist for XML?
                // re: #12967.. a type w/ numeric keys (ie HashMap<usize, V> etc)
                // is going to have a string here, as per JSON spec.
                match s.parse() {
                    Some(f) => Ok(f),
                    None => Err(ExpectedError("Number".to_string(), s)),
                } 
            },
            Xml::Null => Ok(f64::NAN), // FIXME: does this exist for XML?
            value => Err(ExpectedError("Number".to_string(), format!("{}", value)))
        }
    }

    fn read_bool(&mut self) -> DecodeResult<bool> {
        expect!(self.pop(), Boolean)
    }

    fn read_char(&mut self) -> DecodeResult<char> {
        let s = try!(self.read_str());
        {
            let mut it = s.chars();
            match (it.next(), it.next()) {
                // exactly one character
                (Some(c), None) => return Ok(c),
                _ => ()
            }
        }
        Err(ExpectedError("single character string".to_string(), format!("{}", s)))
    }

    fn read_str(&mut self) -> DecodeResult<string::String> {
        expect!(self.pop(), String)
    }

    fn read_enum<T, F>(&mut self, _name: &str, f: F) -> DecodeResult<T> where
        F: FnOnce(&mut Decoder) -> DecodeResult<T>,
    {
        f(self)
    }

    fn read_enum_variant<T, F>(&mut self, names: &[&str],
                               mut f: F) -> DecodeResult<T>
        where F: FnMut(&mut Decoder, usize) -> DecodeResult<T>,
    {
        let name = match self.pop() {
            Xml::String(s) => s,
            Xml::Object(mut o) => {
                let n = match o.remove(&"variant".to_string()) {
                    Some(Xml::String(s)) => s,
                    Some(val) => {
                        return Err(ExpectedError("String".to_string(), format!("{}", val)))
                    }
                    None => {
                        return Err(MissingFieldError("variant".to_string()))
                    }
                };
                match o.remove(&"fields".to_string()) {
                    Some(Xml::Array(l)) => {
                        for field in l.into_iter().rev() {
                            self.stack.push(field);
                        }
                    },
                    Some(val) => {
                        return Err(ExpectedError("Array".to_string(), format!("{}", val)))
                    }
                    None => {
                        return Err(MissingFieldError("fields".to_string()))
                    }
                }
                n
            }
            xml => {
                return Err(ExpectedError("String or Object".to_string(), format!("{}", xml)))
            }
        };
        let idx = match names.iter().position(|n| *n == &name[]) {
            Some(idx) => idx,
            None => return Err(UnknownVariantError(name))
        };
        f(self, idx)
    }

    fn read_enum_variant_arg<T, F>(&mut self, _idx: usize, f: F) -> DecodeResult<T> where
        F: FnOnce(&mut Decoder) -> DecodeResult<T>,
    {
        f(self)
    }

    fn read_enum_struct_variant<T, F>(&mut self, names: &[&str], f: F) -> DecodeResult<T> where
        F: FnMut(&mut Decoder, usize) -> DecodeResult<T>,
    {
        self.read_enum_variant(names, f)
    }


    fn read_enum_struct_variant_field<T, F>(&mut self,
                                         _name: &str,
                                         idx: usize,
                                         f: F)
                                         -> DecodeResult<T> where
        F: FnOnce(&mut Decoder) -> DecodeResult<T>,
    {
        self.read_enum_variant_arg(idx, f)
    }

    fn read_struct<T, F>(&mut self, _name: &str, _len: usize, f: F) -> DecodeResult<T> where
        F: FnOnce(&mut Decoder) -> DecodeResult<T>,
    {
        let value = try!(f(self));
        self.pop();
        Ok(value)
    }

    fn read_struct_field<T, F>(&mut self,
                               name: &str,
                               _idx: usize,
                               f: F)
                               -> DecodeResult<T> where
        F: FnOnce(&mut Decoder) -> DecodeResult<T>,
    {
        let mut obj = try!(expect!(self.pop(), Object));

        let value = match obj.remove(&name.to_string()) {
            None => {
                // Add a Null and try to parse it as an Option<_>
                // to get None as a default value.
                self.stack.push(Xml::Null);
                match f(self) {
                    Ok(x) => x,
                    Err(_) => return Err(MissingFieldError(name.to_string())),
                }
            },
            Some(xml) => {
                self.stack.push(xml);
                try!(f(self))
            }
        };
        self.stack.push(Xml::Object(obj));
        Ok(value)
    }

    fn read_tuple<T, F>(&mut self, tuple_len: usize, f: F) -> DecodeResult<T> where
        F: FnOnce(&mut Decoder) -> DecodeResult<T>,
    {
        self.read_seq(move |d, len| {
            if len == tuple_len {
                f(d)
            } else {
                Err(ExpectedError(format!("Tuple{}", tuple_len), format!("Tuple{}", len)))
            }
        })
    }

    fn read_tuple_arg<T, F>(&mut self, idx: usize, f: F) -> DecodeResult<T> where
        F: FnOnce(&mut Decoder) -> DecodeResult<T>,
    {
        self.read_seq_elt(idx, f)
    }

    fn read_tuple_struct<T, F>(&mut self,
                               _name: &str,
                               len: usize,
                               f: F)
                               -> DecodeResult<T> where
        F: FnOnce(&mut Decoder) -> DecodeResult<T>,
    {
        self.read_tuple(len, f)
    }

    fn read_tuple_struct_arg<T, F>(&mut self,
                                   idx: usize,
                                   f: F)
                                   -> DecodeResult<T> where
        F: FnOnce(&mut Decoder) -> DecodeResult<T>,
    {
        self.read_tuple_arg(idx, f)
    }

    fn read_option<T, F>(&mut self, mut f: F) -> DecodeResult<T> where
        F: FnMut(&mut Decoder, bool) -> DecodeResult<T>,
    {
        match self.pop() {
            Xml::Null => f(self, false),
            value => { self.stack.push(value); f(self, true) }
        }
    }

    fn read_seq<T, F>(&mut self, f: F) -> DecodeResult<T> where
        F: FnOnce(&mut Decoder, usize) -> DecodeResult<T>,
    {
        let array = try!(expect!(self.pop(), Array));
        let len = array.len();
        for v in array.into_iter().rev() {
            self.stack.push(v);
        }
        f(self, len)
    }

    fn read_seq_elt<T, F>(&mut self, _idx: usize, f: F) -> DecodeResult<T> where
        F: FnOnce(&mut Decoder) -> DecodeResult<T>,
    {
        f(self)
    }

    fn read_map<T, F>(&mut self, f: F) -> DecodeResult<T> where
        F: FnOnce(&mut Decoder, usize) -> DecodeResult<T>,
    {
        let obj = try!(expect!(self.pop(), Object));
        let len = obj.len();
        for (key, value) in obj.into_iter() {
            self.stack.push(value);
            self.stack.push(Xml::String(key));
        }
        f(self, len)
    }

    fn read_map_elt_key<T, F>(&mut self, _idx: usize, f: F) -> DecodeResult<T> where
       F: FnOnce(&mut Decoder) -> DecodeResult<T>,
    {
        f(self)
    }
    fn read_map_elt_val<T, F>(&mut self, _idx: usize, f: F) -> DecodeResult<T> where
       F: FnOnce(&mut Decoder) -> DecodeResult<T>,
    {
        f(self)
    }

    fn error(&mut self, err: &str) -> DecoderError {
        ApplicationError(err.to_string())
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

struct FormatShim<'a, 'b: 'a> {
    inner: &'a mut fmt::Formatter<'b>,
}

impl<'a, 'b> fmt::Writer for FormatShim<'a, 'b> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.inner.write_str(s)
    }
}

impl fmt::String for Xml {
    /// Encodes an XML value into a string
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut shim = FormatShim { inner: f };
        let mut encoder = Encoder::new(&mut shim);
        self.encode(&mut encoder)
    }
}

impl<'a, T: Encodable> fmt::String for AsXml<'a, T> {
    /// Encodes an XML value into a string
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut shim = FormatShim { inner: f };
        let mut encoder = Encoder::new(&mut shim);
        self.inner.encode(&mut encoder)
    }
}

/*
impl FromStr for Xml {
    fn from_str(s: &str) -> Option<Xml> {
        Xml::from_str(s).ok()
    }
}
*/

#[cfg(test)]
mod tests {

}
