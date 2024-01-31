use std::io::{Cursor, Read};
use lz4_flex::decompress_into;
use crate::cbor_decoder::CborDecoder;
use crate::types::{Attribute, AttributeType, DataType, Event, FTR, Timescale, Transaction, TxBlock, TxGenerator, TxRelation, TxStream};
use crate::types::DataType::*;

const INFO_CHUNK: u64 = 6;
const DICTIONARY_CHUNK_UNCOMP: u64 = 8;
const DICTIONARY_CHUNK_COMP: u64 = 9;
const DIRECTORY_CHUNK_UNCOMP: u64 = 10;
const DIRECTORY_CHUNK_COMP: u64 = 11;
const TX_BLOCK_CHUNK_UNCOMP: u64 = 12;
const TX_BLOCK_CHUNK_COMP: u64 = 13;
const RELATIONSHIP_CHUNK_UNCOMP: u64 = 14;
const RELATIONSHIP_CHUNK_COMP: u64 = 15;

const STREAM: u64 = 16;
const GENERATOR: u64 = 17;

const EVENT_TAG: u64 = 6;
const BEGIN_TAG: u64 = 7;
const RECORD_TAG: u64 = 8;
const END_TAG: u64 = 9;

pub struct FtrParser<'a> {
    ftr: &'a mut FTR,
}

impl<'a> FtrParser<'a>{

    pub fn new(ftr: &'a mut FTR) -> FtrParser<'a>{

        Self {ftr}
    }

    pub fn load<R: Read>(&mut self, file: R){
        let cbor_decoder = CborDecoder::new(file);
        Self::parse_input(self, cbor_decoder);

    }

    //TODO change to work with buffered readers
    fn parse_input<R: Read>(&mut self, mut cbor_decoder: CborDecoder<R>) {
        let tag = cbor_decoder.read_tag();
        if tag != 55799 {
            panic!("Not a valid ftr file")
        }
        let array_length = cbor_decoder.read_array_length();
        if array_length != -1 {
            panic!()
        }
        let mut next= cbor_decoder.peek();
        while next.is_ok() && next.unwrap() != 0xff {
            let tag = cbor_decoder.read_tag();

            match tag as u64 {
                INFO_CHUNK => {
                    let mut cbd: CborDecoder<Cursor<Vec<u8>>> = CborDecoder::new(Cursor::new(cbor_decoder.read_byte_string()));
                    let size = cbd.read_array_length();
                    if size != 2 {
                        panic!()
                    }

                    let time_scale = cbd.read_int();
                    self.ftr.time_scale = Timescale::get_timescale(time_scale);

                    let epoch_tag = cbd.read_tag();
                    if epoch_tag != 1 {
                        panic!()
                    }
                    cbd.read_int(); // creation time
                }
                DICTIONARY_CHUNK_UNCOMP => {
                    let mut cbd: CborDecoder<Cursor<Vec<u8>>> = CborDecoder::new(Cursor::new(cbor_decoder.read_byte_string()));
                    Self::parse_dict(self, &mut cbd);
                }

                DICTIONARY_CHUNK_COMP => {
                    let len = cbor_decoder.read_array_length();
                    if len != 2 {
                        panic!()
                    }
                    let size = cbor_decoder.read_int(); // uncompressed size
                    let bytes = cbor_decoder.read_byte_string();

                    let mut buf = vec![0u8; size as usize];
                    decompress_into(bytes.as_slice(), &mut buf).expect("");

                    Self::parse_dict(self, &mut CborDecoder::new(Cursor::new(buf)));
                }

                DIRECTORY_CHUNK_UNCOMP => {
                    let mut cbd = CborDecoder::new(Cursor::new(cbor_decoder.read_byte_string()));
                    Self::parse_dir(self, &mut cbd);
                }

                DIRECTORY_CHUNK_COMP => {
                    let size = cbor_decoder.read_array_length();
                    if size != 2 {
                        panic!()
                    }

                    let uncomp_size: usize = cbor_decoder.read_int() as usize;
                    let mut buf = vec![0u8; uncomp_size];
                    let bytes = cbor_decoder.read_byte_string();
                    decompress_into(bytes.as_slice(), &mut buf).expect("");

                    Self::parse_dir(self, &mut CborDecoder::new(Cursor::new(buf)));
                  }

                TX_BLOCK_CHUNK_UNCOMP => {
                    let len = cbor_decoder.read_array_length();
                    if len != 4 {
                        panic!()
                    }

                    let stream_id = cbor_decoder.read_int();
                    let start_time = cbor_decoder.read_int(); // start time of block
                    let end_time = cbor_decoder.read_int();

                    let mut tx_block = TxBlock{
                        stream_id,
                        start_time,
                        end_time,
                        transactions: vec![],
                    };

                    Self::parse_tx_block(self, &mut CborDecoder::new(Cursor::new(cbor_decoder.read_byte_string())), &mut tx_block);
                    self.ftr.tx_blocks.push(tx_block);

                }

                TX_BLOCK_CHUNK_COMP => {
                    let len = cbor_decoder.read_array_length();
                    if len != 5 {
                        panic!()
                    }

                    let stream_id = cbor_decoder.read_int();
                    let start_time = cbor_decoder.read_int(); // start time of block
                    let end_time = cbor_decoder.read_int();
                    let uncomp_size = cbor_decoder.read_int();

                    let mut tx_block = TxBlock{
                        stream_id,
                        start_time,
                        end_time,
                        transactions: vec![],
                    };

                    let mut buf = vec![0u8; uncomp_size as usize];
                    let bytes = cbor_decoder.read_byte_string();
                    decompress_into(bytes.as_slice(), &mut buf).expect("");

                    Self::parse_tx_block(self, &mut CborDecoder::new(Cursor::new(buf)), &mut tx_block);
                    self.ftr.tx_blocks.push(tx_block);
                }

                RELATIONSHIP_CHUNK_UNCOMP => {
                    let mut cbd = CborDecoder::new(Cursor::new(cbor_decoder.read_byte_string()));
                    Self::parse_rel(self, &mut cbd);
                }

                RELATIONSHIP_CHUNK_COMP => {
                    let len = cbor_decoder.read_array_length();
                    if len != 2 {
                        panic!()
                    }
                    let uncomp_size = cbor_decoder.read_int();
                    let mut buf = vec![0u8; uncomp_size as usize];
                    let bytes = cbor_decoder.read_byte_string();
                    decompress_into(bytes.as_slice(), &mut buf).expect("");

                    Self::parse_rel(self, &mut CborDecoder::new(Cursor::new(buf)));
                   }

                _ => {panic!("Should never happen")}
            }

            next = cbor_decoder.peek();
        }
    }

    fn parse_dict<R: Read>(&mut self, cbd: &mut CborDecoder<R>) {
        let size = cbd.read_map_length();

        for _i in 0..size {
            let idx = cbd.read_int();
            self.ftr.str_dict.insert(idx, cbd.read_text_string());
        }


    }

    fn parse_dir<R: Read>(&mut self, cbd: &mut CborDecoder<R>){
        let size = cbd.read_array_length();
        if size < 0 {
            let mut next_dir = cbd.peek();
            while next_dir.is_ok() && next_dir.unwrap() != 0xff {
                Self::parse_dir_entry(self, cbd);

                next_dir = cbd.peek();
            }

        }else {
            for _i in 1..size {
                Self::parse_dir_entry(self, cbd);
            }
        }
    }


    fn parse_dir_entry<R: Read>(&mut self, cbd: &mut CborDecoder<R>) {
        let dir_tag = cbd.read_tag();
        if dir_tag == STREAM as i64{
            let len = cbd.read_array_length();
            if len != 3 {
                panic!()
            }
            let stream_id = cbd.read_int();

            let name_id = cbd.read_int();
            let name = self.ftr.str_dict.get(&name_id).expect("");

            let kind_id = cbd.read_int();
            let kind = self.ftr.str_dict.get(&kind_id).expect("");

            self.ftr.tx_streams.push(TxStream{id: stream_id, name: name.clone(), kind: kind.clone()});

        } else if dir_tag == GENERATOR as i64{
            let len = cbd.read_array_length();
            if len != 3 {
                panic!()
            }
            let gen_id = cbd.read_int();

            let name_id = cbd.read_int();

            let name = self.ftr.str_dict.get(&name_id).expect("");

            let stream_id = cbd.read_int();

            self.ftr.tx_generators.push(TxGenerator{id: gen_id, name: name.clone(), stream_id});

        }
    }

    fn parse_tx_block<R: Read>(&mut self, cbd: &mut CborDecoder<R>, tx_block: &mut TxBlock) {
        let size = cbd.read_array_length();
        if size != -1 {
            panic!()
        }

        let mut next_tx = cbd.peek();
        while next_tx.is_ok() && next_tx.unwrap() != 0xff {

            let arr_len = cbd.read_array_length();



            let mut event = Event::new();
            let mut attributes = vec![Attribute::new(); 0];

            for _i in 0..arr_len {
                let tag = cbd.read_tag();

                match tag  as u64{
                    EVENT_TAG => {
                        let event_len = cbd.read_array_length();
                        if event_len != 4 {
                            panic!()
                        }
                        let tx_id = cbd.read_int();
                        let gen_id = cbd.read_int();
                        let start_time = cbd.read_int();
                        let end_time = cbd.read_int();
                        let new_event = Event{
                            tx_id,
                            gen_id,
                            start_time,
                            end_time,
                        };
                        event = new_event;
                    }
                    BEGIN_TAG => {
                        let len = cbd.read_array_length();
                        if len != 3 {
                            panic!()
                        }
                        let name_id = cbd.read_int();
                        let data_type = cbd.read_int();
                        let value = cbd.read_int(); // Placeholder TODO should be differentiated based on data type
                        /*let value = match data_type {
                            0 => cbd.read_int()

                        };*/
                        let new_begin = Attribute{
                            kind: AttributeType::BEGIN,
                            name: self.ftr.str_dict.get(&name_id).unwrap().clone(),
                            data_type: int2data_type(data_type),
                            value,
                        };

                        attributes.push(new_begin);
                    }
                    RECORD_TAG => {
                        let len = cbd.read_array_length();
                        if len != 3 {
                            panic!()
                        }
                        let name_id = cbd.read_int();
                        let data_type = cbd.read_int();
                        let value = cbd.read_int(); // Placeholder TODO should be differentiated based on data type
                        /*let value = match data_type {
                            0 => cbd.read_int()

                        };*/

                        let new_record = Attribute{
                            kind: AttributeType::RECORD,
                            name: self.ftr.str_dict.get(&name_id).unwrap().clone(),
                            data_type: int2data_type(data_type),
                            value,
                        };

                        attributes.push(new_record);
                    }
                    END_TAG => {
                        let len = cbd.read_array_length();
                        if len != 3 {
                            panic!()
                        }
                        let name_id = cbd.read_int();
                        let data_type = cbd.read_int();
                        let value = cbd.read_int(); // Placeholder TODO should be differentiated based on data type
                        /*let value = match data_type {
                            0 => cbd.read_int()

                        };*/
                        let new_end = Attribute{
                            kind: AttributeType::END,
                            name: self.ftr.str_dict.get(&name_id).unwrap().clone(),
                            data_type: int2data_type(data_type),
                            value,
                        };

                        attributes.push(new_end);
                    }
                    _ => {panic!("Should never happen")}
                }

            }

            let tx = Transaction{
                event,
                attributes,
            };
            tx_block.transactions.push(tx);

            next_tx = cbd.peek();

        }

    }

    fn parse_rel<R: Read>(&mut self, cbd: &mut CborDecoder<R>) {
        let size = cbd.read_array_length();
        if size != -1 {
            panic!()
        }

        let mut next_rel = cbd.peek();
        while next_rel.is_ok() && next_rel.unwrap() != 0xff {
            let sz = cbd.read_array_length();
            if sz != 5 && sz != 3 {
                panic!()
            }
            let type_id = cbd.read_int();
            let from_tx_id = cbd.read_int();
            let to_tx_id = cbd.read_int();
            let from_stream_id = if sz > 3 {cbd.read_int()} else {-1};
            let to_stream_id = if sz > 3 {cbd.read_int()} else {-1};
            let rel_name = self.ftr.str_dict.get(&type_id).unwrap();

            let tx_relation = TxRelation{
                name: rel_name.clone(),
                source_tx_id: from_tx_id,
                sink_tx_id: to_tx_id,
                source_stream_id: from_stream_id,
                sink_stream_id: to_stream_id,
            };
            self.ftr.tx_relations.push(tx_relation);
            next_rel = cbd.peek();
        }
    }

}

fn int2data_type(input: i64) -> DataType{
    match input {
        0 => BOOLEAN,
        1 => ENUMERATION,
        2 => INTEGER,
        3 => UNSIGNED,
        4 => FLOATING_POINT_NUMBER,
        5 => BIT_VECTOR,
        6 => LOGIC_VECTOR,
        7 => FIXED_POINT_INTERGER,
        8 => UNSIGNED_FIXED_POINT_INTEGER,
        9 => POINTER,
        10 => STRING,
        11 => TIME,
        _ => NONE,
    }
}



