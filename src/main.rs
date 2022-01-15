#![allow(dead_code)]
#[macro_use] extern crate lazy_static;


mod engine;
mod game;
mod interface;
mod rules;
mod testing;
mod util;


use interface::cli::Interface;


fn main() {
    let mut interface = Interface::new();
    interface.init();
}
