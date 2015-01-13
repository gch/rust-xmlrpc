// Copyright 2014-2015 Galen Clark Haynes
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// Rust XML-RPC library

use hyper;
use std::string;

pub struct Client {
    url: string::String,
}

impl Client {
    pub fn new(s: &str) -> Client {
        Client { url: s.to_string() }
    }

    pub fn remote_call(&self, request: super::Request) -> () {
        let mut http_client = hyper::Client::new();
        let mut result = http_client.post(self.url.as_slice())
            .body(request.body.as_slice()) // FIXME: use to_xml() somehow?
            .send();
        let response = Some(result.ok().unwrap().read_to_string().unwrap());
        println!("{}", response.unwrap());
        // None // FIXME: actually return response
    }
}
