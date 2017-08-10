extern crate devd_rs;

use devd_rs::*;

fn main() {
    let mut ctx = Context::new().unwrap();
    loop {
        println!("{:?}", ctx.wait_for_event().unwrap());
    }
}
