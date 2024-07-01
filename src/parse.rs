use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, Cursor, Read, Seek, SeekFrom};
use num_bigint::BigInt;
use crate::cbor_decoder::CborDecoder;
use crate::ftr_parser::FtrParser;
use crate::types::{FTR, Timescale};



/// The function you probably want to call first.
/// Parses the file with the given name and returns a FTR variable with all streams, generators and relations already accessible.
/// However, it does not yet load the transactions themselves into memory. This can be done with 'load_stream_into_memory()'.
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

pub fn read_from_bytes(bytes: Vec<u8>) -> color_eyre::Result<FTR>{

    let mut ftr = FTR{
        time_scale: Timescale::None,
        str_dict: HashMap::new(),
        tx_streams: HashMap::new(),
        max_timestamp: BigInt::from(0),
        tx_generators: HashMap::new(),
        tx_relations: vec![],
        file_name: "".to_string(),
    };
    let mut ftr_parser = FtrParser::new(&mut ftr);

    ftr_parser.load(Cursor::new(bytes))?;

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

pub fn is_ftr<R: Read + Seek>(input: &mut R) -> bool {
    let mut cbor_decoder = CborDecoder::new(input);
    let tag = cbor_decoder.read_tag();
    cbor_decoder.input_stream.seek(SeekFrom::Start(0)).unwrap();
    tag == 55799
}
