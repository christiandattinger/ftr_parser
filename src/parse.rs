use std::collections::HashMap;
use std::fs::File;
use std::io::{Cursor, SeekFrom};
use std::path::PathBuf;
use num_bigint::BigInt;

use crate::cbor_decoder::CborDecoder;
use crate::ftr_parser::FtrParser;
use crate::types::{FTR, Timescale};

/// The function you probably want to call first.
/// Parses the file with the given name and returns a FTR variable with all streams, generators and relations already accessible.
/// However, it does not yet load the transactions themselves into memory. This can be done with 'load_stream_into_memory()'.
pub fn parse_ftr(file_name: PathBuf) -> color_eyre::Result<FTR>{

    let mut ftr = FTR{
        time_scale: Timescale::None,
        str_dict: HashMap::new(),
        tx_streams: HashMap::new(),
        max_timestamp: BigInt::from(0),
        tx_generators: HashMap::new(),
        tx_relations: vec![],
        path: Some(file_name.clone()),
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
        path: None,
    };
    let mut ftr_parser = FtrParser::new(&mut ftr);

    ftr_parser.load(Cursor::new(bytes))?;

    Ok(ftr)
}

pub fn is_ftr<R: std::io::Read + std::io::Seek>(input: &mut R) -> bool {
    let mut cbor_decoder = CborDecoder::new(input);
    let tag = cbor_decoder.read_tag();
    cbor_decoder.input_stream.seek(SeekFrom::Start(0)).unwrap();
    match tag {
        Ok(tag) => tag == 55799,
        Err(_) => false,
    }
}

