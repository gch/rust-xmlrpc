// Copyright 2014-2015 Galen Clark Haynes
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// Rust XML-RPC library

use std::string;
use rustc_serialize::{Encodable};

pub struct Request {
    pub method: string::String,
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
