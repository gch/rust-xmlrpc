#![allow(unstable)]

extern crate xmlrpc;

fn main() {
    let master = std::os::getenv("ROS_MASTER_URI").unwrap();
    let client = xmlrpc::Client::new(master.as_slice());
    let mut request = xmlrpc::Request::new("getSystemState");
    request = request.argument(&"/").finalize();
    let response = client.remote_call(&request);//.unwrap();
}
