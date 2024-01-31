use std::collections::HashMap;
use std::io::Read;
use crate::ftr_parser::FtrParser;
use crate::types::{FTR, Timescale};

pub fn parse_ftr<R: Read>(reader: R) -> Result<FTR, String>{

    let mut ftr = FTR{
        time_scale: Timescale::Unit,
        str_dict: HashMap::new(),
        tx_streams: vec![],
        tx_generators: vec![],
        tx_blocks: vec![],
        tx_relations: vec![],
    };
    let mut ftr_parser = FtrParser::new(&mut ftr);

    ftr_parser.load(reader);

    Ok(ftr)


}