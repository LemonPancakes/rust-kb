#![feature(test)]

extern crate rust_kb;
extern crate test;
//extern crate pest_parser;

use rust_kb::kb::parser::kb;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use test::Bencher;

use rust_kb::pest_parser_bk::parse;

// ~6s
#[bench]
fn nom_parse(b: &mut Bencher) {
    let file = fs::read("test/large.kb").expect("file not found");

    b.iter(|| kb(&file).unwrap());
}

// ~10s
#[bench]
fn pest_parse(b: &mut Bencher) {
    let file = fs::read_to_string("test/large.kb").expect("Unable to read the file");
    b.iter(|| parse(&file));
}
