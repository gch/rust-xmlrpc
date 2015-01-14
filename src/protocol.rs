// Copyright 2014-2015 Galen Clark Haynes
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// Rust XML-RPC library

use std::string;
use rustc_serialize::{Encodable,Decodable};

pub struct Request {
    pub method: string::String,
    pub body: string::String,
}

pub struct Response {
    pub body: string::String,
}

impl Request {
    pub fn new(method: &str) -> Request {
        Request {
            method: method.to_string(),
            body: format!("\
            <?xml version=\"1.0\"?>\
            <methodCall><methodName>{}</methodName>\
                <params>", method),
        }
    }

    pub fn argument<T: Encodable>(mut self, object: &T) -> Request {
        let append_body = format!("<param>{}</param>", super::encode(object));
        self.body = self.body + append_body.as_slice();
        self
    }

    pub fn finalize(mut self) -> Request {
        self.body = self.body + "</params></methodCall>";
        self
    }

}

impl Response {
    pub fn new(body: &str) -> Response {
        Response {
            body: body.to_string(),
        }
    }

    pub fn result<T: Decodable>(&self, idx: usize) -> Option<T> {
        // FIXME: use idx
        let resp = self.body.clone(); // FIXME: no need to clone
        let val0 = "<params>\n<param>\n<value>"; // FIXME: use xml-rs rather than manual search
        let idx0 = resp.find_str(val0).unwrap() + val0.len();
        let val1 = "</value>\n</param>\n</params>";
        let idx1 = resp.find_str(val1).unwrap();
        let object: T = super::decode(resp.slice(idx0,idx1)).unwrap();
        Some(object)
    }
}
