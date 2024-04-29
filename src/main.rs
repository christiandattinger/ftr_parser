mod ftr_parser;
mod cbor_decoder;
mod types;
mod parse;

use crate::parse::{drop_stream_from_memory, load_stream_into_memory, parse_ftr};

fn main() -> color_eyre::Result<()>{

    let comp = false;
    let file = if comp {
        String::from("my_db_c.ftr")
    }else {
        String::from("my_db.ftr")
    };

    //let file = String::from("my_db_invalid.ftr");

    let mut ftr = parse_ftr(file)?;

    println!("Timescale: {:?}", ftr.time_scale);

    println!("Max timestamp: {:?}", ftr.max_timestamp);

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

    println!("Generators: ");
    for gen in &ftr.tx_generators {
        println!("Generator {:?} {:?}: ", gen.0, gen.1.name.clone());
        for tx in &gen.1.transactions {
            println!("  {:?}", tx);
        }
    }
    println!();

    println!("Relations: ");
    for rel in &ftr.tx_relations {
        println!("{:?}", rel);
    }

    load_stream_into_memory(&mut ftr, 1)?;
    load_stream_into_memory(&mut ftr, 2)?;

    println!("Streams: ");
    for stream in &ftr.tx_streams {
        println!("{:?}", stream);
    }
    println!();

    println!("Generators: ");
    for gen in &ftr.tx_generators {
        println!("Generator {:?} {:?}: ", gen.0, gen.1.name.clone());
        for tx in &gen.1.transactions {
            println!("  {:?}", tx);
        }

    }
    println!();

    drop_stream_from_memory(&mut ftr, 1);

    println!("Streams: ");
    for stream in &ftr.tx_streams {
        println!("{:?}", stream);
    }
    println!();

    println!("Generators: ");
    for gen in &ftr.tx_generators {
        println!("Generator {:?} {:?}: ", gen.0, gen.1.name.clone());
        for tx in &gen.1.transactions {
            println!("  {:?}", tx);
        }
    }
    println!();

    Ok(())

}



