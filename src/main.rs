#![allow(dead_code)]
#[macro_use] extern crate lazy_static;


use interface::cli::Interface;



mod engine;
mod game;
mod interface;
mod rules;
mod util;


fn main() {
    let mut interface = Interface::new();
    interface.init();
}
