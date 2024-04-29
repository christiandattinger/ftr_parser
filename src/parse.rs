use std::collections::HashMap;
use std::fs::File;
use num_bigint::BigInt;
use crate::ftr_parser::FtrParser;
use crate::types::{FTR, Timescale};



// The function you probbably want to call first.
// Parses the file with the given name and returns a FTR variable with all streams, generators and relations already accessible.
// However, it does not yet load the transactions themselves into memory. This can be done with 'load_stream_into_memory()'.
pub fn parse_ftr(file_name: String) -> color_eyre::Result<FTR>{

    let mut ftr = FTR{
        time_scale: Timescale::None,
        str_dict: HashMap::new(),
        tx_streams: HashMap::new(),
        max_timestamp: BigInt::from(0),
        tx_generators: HashMap::new(),
        tx_relations: vec![],
        file_name: file_name.clone(),
    };
    let mut ftr_parser = FtrParser::new(&mut ftr);

    let reader = File::open(file_name)?;

    ftr_parser.load(reader)?;

    Ok(ftr)
}

// Takes a stream id and loads all associated transactions into memory
pub fn load_stream_into_memory(ftr: &mut FTR, stream_id: usize) -> color_eyre::Result<()>{
    let mut ftr_parser = FtrParser::new(ftr);

    ftr_parser.load_transactions(stream_id)?;

    Ok(())
}

// drops all transactions from this stream from memory, but the stream itself doesn't get deleted
pub fn drop_stream_from_memory(ftr: &mut FTR, stream_id: usize) {
    for gen_id in &ftr.tx_streams.get(&stream_id).expect("").generators {
        ftr.tx_generators.get_mut(gen_id).unwrap().transactions = vec![];
    }
}
