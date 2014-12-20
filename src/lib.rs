// Copyright 2014 Galen Clark Haynes
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// Rust XML-RPC library
// Copyright (c) 2014 Galen Clark Haynes

// Derived from Rust JSON library
// Copyright (c) 2011 Google Inc.

#![crate_name = "xmlrpc"]
#![comment = "Rust XML-RPC library"]
#![license = "BSD"]
#![crate_type = "rlib"]
#![crate_type = "dylib"]

#![forbid(non_camel_case_types)]
#![allow(missing_docs)]

#![feature(slicing_syntax)]

/*!
XML-RPC library, including both serialization and remote procedure calling

# What is XML-RPC?

Documentation to be written ... (follow example in json.rs)

Basic documentation found on Wikipedia
    http://en.wikipedia.org/wiki/XML-RPC

Full specification of the XML-RPC protocol is found here:
    http://xmlrpc.scripting.com/spec.html

Additional errata and hints can be found here:
    http://effbot.org/zone/xmlrpc-errata.htm
*/

extern crate serialize;

use std::collections::{HashMap, TreeMap};
use std::{io, fmt, mem, str};
use std::io::MemWriter;
use serialize::{Encodable, Decodable};
use serialize::Encoder as SerializeEncoder;
use std::string;
use std::mem::{swap, transmute};

/// Represents an XML-RPC data value
#[deriving(Clone, PartialEq, PartialOrd)]
pub enum Xml {
     I64(i64),
     U64(u64),
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
pub type Object = TreeMap<string::String, Xml>;

/// Shortcut function to encode a `T` into a XML-RPC `String`
pub fn encode<'a, T: Encodable<Encoder<'a>, io::IoError>>(object: &T) -> string::String {
    let buff = Encoder::buffer_encode(object);
    string::String::from_utf8(buff).unwrap()
}

pub type EncodeResult = io::IoResult<()>;

pub fn escape_bytes(wr: &mut io::Writer, bytes: &[u8]) -> Result<(), io::IoError> {
    // FIXME:
    //  replace < with &lt;
    //  replace > with &gt;
    //  replace & with &amp;
    wr.write(bytes[0..])
}

fn escape_str(writer: &mut io::Writer, v: &str) -> Result<(), io::IoError> {
    escape_bytes(writer, v.as_bytes())
}

fn escape_char(writer: &mut io::Writer, v: char) -> Result<(), io::IoError> {
    let mut buf = [0, .. 4];
    let len = v.encode_utf8(&mut buf).unwrap();
    escape_bytes(writer, buf[mut ..len])
}

/// A structure for implementing serialization to XML-RPC.
pub struct Encoder<'a> {
    writer: &'a mut (io::Writer+'a),
}

impl<'a> Encoder<'a> {
    /// Creates a new XML-RPC encoder whose output will be written to the writer
    /// specified.
    pub fn new(writer: &'a mut io::Writer) -> Encoder<'a> {
        Encoder { writer: writer }
    }

    /// Encode the specified object into a buffer [u8]
    pub fn buffer_encode<T:Encodable<Encoder<'a>, io::IoError>>(object: &T) -> Vec<u8>  {
        //Serialize the object in a string using a writer
        let mut m = MemWriter::new();
        // FIXME(14302) remove the transmute and unsafe block.
        unsafe {
            let mut encoder = Encoder::new(&mut m as &mut io::Writer);
            // MemWriter never Errs
            let _ = object.encode(transmute(&mut encoder));
        }
        m.unwrap()
    }
}

impl<'a> SerializeEncoder<io::IoError> for Encoder<'a> {
    fn emit_nil(&mut self) -> EncodeResult { write!(self.writer, "<nil/>") }
    fn emit_uint(&mut self, v: uint) -> EncodeResult { self.emit_i32(v as i32) }
    fn emit_u64(&mut self, v: u64) -> EncodeResult { self.emit_i32(v as i32) }
    fn emit_u32(&mut self, v: u32) -> EncodeResult { self.emit_i32(v as i32) }
    fn emit_u16(&mut self, v: u16) -> EncodeResult { self.emit_i32(v as i32) }
    fn emit_u8(&mut self, v: u8) -> EncodeResult { self.emit_i32(v as i32) }

    fn emit_int(&mut self, v: int) -> EncodeResult { self.emit_i32(v as i32) }
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
                            _id: uint,
                            cnt: uint,
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


    fn emit_enum_variant_arg<F>(&mut self, idx: uint, f: F) -> EncodeResult where
        F: FnOnce(&mut Encoder<'a>) -> EncodeResult,
    {
        if idx != 0 {
            try!(write!(self.writer, ","));
        }
        f(self)
    }

    fn emit_enum_struct_variant<F>(&mut self,
                                   name: &str,
                                   id: uint,
                                   cnt: uint,
                                   f: F) -> EncodeResult where
        F: FnOnce(&mut Encoder<'a>) -> EncodeResult,
    {
        self.emit_enum_variant(name, id, cnt, f)
    }

    fn emit_enum_struct_variant_field<F>(&mut self,
                                         _: &str,
                                         idx: uint,
                                         f: F) -> EncodeResult where
        F: FnOnce(&mut Encoder<'a>) -> EncodeResult,
    {
        self.emit_enum_variant_arg(idx, f)
    }

    fn emit_struct<F>(&mut self, _: &str, _: uint, f: F) -> EncodeResult where
        F: FnOnce(&mut Encoder<'a>) -> EncodeResult,
    {
        try!(write!(self.writer, "<struct>"));
        try!(f(self));
        write!(self.writer, "</struct>")
    }

    fn emit_struct_field<F>(&mut self, name: &str, idx: uint, f: F) -> EncodeResult where
        F: FnOnce(&mut Encoder<'a>) -> EncodeResult,
    {
        try!(write!(self.writer, "<member>"));
        try!(write!(self.writer, "<name>{}</name>", name)); // FIXME: encode str?
        try!(write!(self.writer, "<value>"));
        try!(f(self));
        try!(write!(self.writer, "</value>"));
        write!(self.writer, "</member>")
    }

    fn emit_tuple<F>(&mut self, len: uint, f: F) -> EncodeResult where
        F: FnOnce(&mut Encoder<'a>) -> EncodeResult,
    {
        self.emit_seq(len, f)
    }
    fn emit_tuple_arg<F>(&mut self, idx: uint, f: F) -> EncodeResult where
        F: FnOnce(&mut Encoder<'a>) -> EncodeResult,
    {
        self.emit_seq_elt(idx, f)
    }

    fn emit_tuple_struct<F>(&mut self, _name: &str, len: uint, f: F) -> EncodeResult where
        F: FnOnce(&mut Encoder<'a>) -> EncodeResult,
    {
        self.emit_seq(len, f)
    }
    fn emit_tuple_struct_arg<F>(&mut self, idx: uint, f: F) -> EncodeResult where
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

    fn emit_seq<F>(&mut self, _len: uint, f: F) -> EncodeResult where
        F: FnOnce(&mut Encoder<'a>) -> EncodeResult,
    {
        try!(write!(self.writer, "<array><data>"));
        try!(f(self));
        write!(self.writer, "</data></array>")
    }

    fn emit_seq_elt<F>(&mut self, idx: uint, f: F) -> EncodeResult where
        F: FnOnce(&mut Encoder<'a>) -> EncodeResult,
    {
        try!(write!(self.writer, "<value>"));
        try!(f(self));
        write!(self.writer, "</value>")
    }

    fn emit_map<F>(&mut self, _len: uint, f: F) -> EncodeResult where
        F: FnOnce(&mut Encoder<'a>) -> EncodeResult,
    {
        Ok(())
        // FIXME: this is JSON source
        //try!(write!(self.writer, "{{"));
        //try!(f(self));
        //write!(self.writer, "}}")
    }

    fn emit_map_elt_key<F>(&mut self, idx: uint, mut f: F) -> EncodeResult where
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

    fn emit_map_elt_val<F>(&mut self, _idx: uint, f: F) -> EncodeResult where
        F: FnOnce(&mut Encoder<'a>) -> EncodeResult,
    {
        Ok(())
        //try!(write!(self.writer, ":"));
        //f(self)
    }
}


impl<E: SerializeEncoder<S>, S> Encodable<E, S> for Xml {
    fn encode(&self, e: &mut E) -> Result<(), S> {
        match *self {
            Xml::I64(v) => v.encode(e),
            Xml::U64(v) => v.encode(e),
            Xml::F64(v) => v.encode(e),
            Xml::String(ref v) => v.encode(e),
            Xml::Boolean(v) => v.encode(e),
            Xml::Array(ref v) => v.encode(e),
            Xml::Object(ref v) => v.encode(e),
            Xml::Null => e.emit_nil(),
            _ => Ok(()), // FIXME: add other types
        }
    }
}


#[cfg(test)]
mod tests {

}
