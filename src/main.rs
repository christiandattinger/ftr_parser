mod ftr_parser;
mod cbor_decoder;
mod types;

use std::collections::HashMap;
use std::fs::{read};

use crate::ftr_parser::FtrParser;
use crate::types::FTR;


fn main() {

    let comp = false;
    let file = if comp {
        read("my_db_c.ftr").expect("")
    }else {
        read("my_db.ftr").expect("")
    };

    let mut ftr = FTR{
        str_dict: HashMap::new(),
        tx_streams: vec![],
        tx_generators: vec![],
        tx_blocks: vec![],
        tx_relations: vec![],
    };
    let mut ftr_parser = FtrParser::new(&mut ftr);

    ftr_parser.load(file);

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

    println!("Generators");
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
    }


}



