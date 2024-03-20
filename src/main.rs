mod ftr_parser;
mod cbor_decoder;
mod types;
mod parse;

use std::fs::File;
use crate::parse::parse_ftr;

fn main() {

    let comp = true;
    let file = if comp {
        File::open("my_db_c.ftr").unwrap()
    }else {
        File::open("my_db.ftr").unwrap()
    };

    let ftr = parse_ftr(file).unwrap();

    println!("Timescale: {:?}", ftr.time_scale);

    println!("Dictionary: ");
    for entry in &ftr.str_dict {
        println!("key: {}, value: {}", entry.0, entry.1);
    }
    println!();

    println!("Streams: ");
    for stream in &ftr.tx_streams {
        println!("{:?}", stream);
    }
    println!();

    /*println!("Generators");
    for gen in &ftr.tx_generators {
        println!("{:?}", gen);
    }
    println!();

    println!("Transactions: ");
    for tx_block in &ftr.tx_blocks {
        for tx in &tx_block.transactions {
            println!("{:?}", tx);
        }
    }
    println!();


    println!("Relations: ");
    for tx_relation in &ftr.tx_relations {
        println!("{:?}", tx_relation);
    }*/


}



