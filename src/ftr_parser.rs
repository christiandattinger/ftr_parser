use std::fs::File;
use std::io::{Cursor, Read, Seek, SeekFrom};

use color_eyre::eyre::bail;
use lz4_flex::decompress_into;
use num_bigint::BigInt;

use crate::cbor_decoder::CborDecoder;
use crate::types::{Attribute, AttributeType, DataType, Event, FTR, Timescale, Transaction, TxGenerator, TxRelation, TxStream};

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

const BOOLEAN: u8 = 0;
const ENUMERATION: u8 = 1;
const INTEGER: u8 = 2;
const UNSIGNED: u8 = 3;
const FLOATING_POINT_NUMBER: u8 = 4;
const BIT_VECTOR: u8 = 5;
const LOGIC_VECTOR: u8 = 6;
const FIXED_POINT_INTEGER: u8 = 7;
const UNSIGNED_FIXED_POINT_INTEGER: u8 = 8;
const POINTER: u8 = 9;
const STRING: u8 = 10;
const TIME: u8 = 11;

pub struct FtrParser<'a> {
    ftr: &'a mut FTR,
}

impl <'a> FtrParser<'a>{

    pub fn new(ftr: &'a mut FTR) -> FtrParser<'a>{
        Self {ftr}
    }

    pub(super) fn load<R: Read + Seek>(&mut self, file: R) -> color_eyre::Result<()> {
        let cbor_decoder = CborDecoder::new(file);
        Self::parse_input(self, cbor_decoder)?;
        Ok(())
    }

    //TODO change to work with buffered readers
    //TODO further error handling
    //TODO add missing data types to attribute values
    fn parse_input<R: Read + Seek>(&mut self, mut cbor_decoder: CborDecoder<R>) -> color_eyre::Result<()>{
        let tag = cbor_decoder.read_tag();
        if tag != 55799 {
            bail!("Not a valid FTR file");
        }
        let array_length = cbor_decoder.read_array_length();
        if array_length != -1 {
           bail!("Array does not have indefinite length. Not a valid FTR file!");
        }
        let mut next= cbor_decoder.peek();
        while next.is_ok() && next.unwrap() != 0xff {
            let tag = cbor_decoder.read_tag();

            match tag as u64 {
                INFO_CHUNK => {
                    let mut cbd: CborDecoder<Cursor<Vec<u8>>> = CborDecoder::new(Cursor::new(cbor_decoder.read_byte_string()));
                    let size = cbd.read_array_length();
                    if size != 2 {
                        bail!("Info Chunk has wrong length. Not a valid FTR file!");
                    }

                    let time_scale = cbd.read_int();
                    self.ftr.time_scale = Timescale::get_timescale(time_scale);

                    let epoch_tag = cbd.read_tag();
                    if epoch_tag != 1 {
                        bail!("Wrong epoch tag. Not a valid FTR file!");
                    }
                    let _creation_time = cbd.read_int();
                }
                DICTIONARY_CHUNK_UNCOMP => {
                    let mut cbd: CborDecoder<Cursor<Vec<u8>>> = CborDecoder::new(Cursor::new(cbor_decoder.read_byte_string()));
                    Self::parse_dict(self, &mut cbd);
                }

                DICTIONARY_CHUNK_COMP => {
                    let len = cbor_decoder.read_array_length();
                    if len != 2 {
                        bail!("Dictionary Chunk has wrong size. Not a valid FTR file!");
                    }
                    let size = cbor_decoder.read_int(); // uncompressed size
                    let bytes = cbor_decoder.read_byte_string();

                    let mut buf = vec![0u8; size as usize];
                    decompress_into(bytes.as_slice(), &mut buf)?;

                    Self::parse_dict(self, &mut CborDecoder::new(Cursor::new(buf)));
                }

                DIRECTORY_CHUNK_UNCOMP => {
                    let mut cbd = CborDecoder::new(Cursor::new(cbor_decoder.read_byte_string()));
                    Self::parse_dir(self, &mut cbd)?;
                }

                DIRECTORY_CHUNK_COMP => {
                    let size = cbor_decoder.read_array_length();
                    if size != 2 {
                        bail!("Directory Chunk has wrong size. Not a valid FTR file!");
                    }

                    let uncomp_size: usize = cbor_decoder.read_int() as usize;
                    let mut buf = vec![0u8; uncomp_size];
                    let bytes = cbor_decoder.read_byte_string();
                    decompress_into(bytes.as_slice(), &mut buf)?;

                    Self::parse_dir(self, &mut CborDecoder::new(Cursor::new(buf)))?;
                  }

                TX_BLOCK_CHUNK_UNCOMP => {
                    let len = cbor_decoder.read_array_length();
                    if len != 4 {
                        bail!("Transaction Block Chunk has wrong size. Not a valid FTR file!");
                    }

                    let stream_id = cbor_decoder.read_int() as usize;
                    let _start_time = cbor_decoder.read_int(); // start time of block
                    let end_time = cbor_decoder.read_int(); // end time of block
                    if BigInt::from(end_time) > self.ftr.max_timestamp {
                        self.ftr.max_timestamp = BigInt::from(end_time);
                    }

                    self.ftr.tx_streams.get_mut(&stream_id).unwrap().tx_block_ids.push((cbor_decoder.input_stream.stream_position().expect(""), false));

                    if self.ftr.file_name == "" {
                        let mut cbd = CborDecoder::new(Cursor::new(cbor_decoder.read_byte_string()));
                        Self::parse_tx_block(self, &mut cbd)?;
                    } else {
                        cbor_decoder.skip_byte_string();  // we don't want to load the transactions right now, so we just skip this whole block
                    }
                }

                TX_BLOCK_CHUNK_COMP => {
                    let len = cbor_decoder.read_array_length();
                    if len != 5 {
                        bail!("Transaction Block Chunk has wrong size. Not a valid FTR file!");
                    }

                    let stream_id = cbor_decoder.read_int() as usize;
                    let _start_time = cbor_decoder.read_int(); // start time of block
                    let end_time = cbor_decoder.read_int(); // end time of block

                    if BigInt::from(end_time) > self.ftr.max_timestamp {
                        self.ftr.max_timestamp = BigInt::from(end_time);
                    }

                    self.ftr.tx_streams.get_mut(&stream_id).unwrap().tx_block_ids.push((cbor_decoder.input_stream.stream_position().expect(""), true));
                    let _uncomp_size = cbor_decoder.read_int();

                    if self.ftr.file_name == "" {
                        let mut cbd = CborDecoder::new(Cursor::new(cbor_decoder.read_byte_string()));
                        Self::parse_tx_block(self, &mut cbd)?;
                    } else {
                        cbor_decoder.skip_byte_string();
                    }
                }

                RELATIONSHIP_CHUNK_UNCOMP => {
                    let mut cbd = CborDecoder::new(Cursor::new(cbor_decoder.read_byte_string()));
                    Self::parse_rel(self, &mut cbd)?;
                }

                RELATIONSHIP_CHUNK_COMP => {
                    let len = cbor_decoder.read_array_length();
                    if len != 2 {
                        bail!("Relationship Chunk has wrong size. Not a valid FTR file.");
                    }
                    let uncomp_size = cbor_decoder.read_int();
                    let mut buf = vec![0u8; uncomp_size as usize];
                    let bytes = cbor_decoder.read_byte_string();
                    decompress_into(bytes.as_slice(), &mut buf)?;

                    Self::parse_rel(self, &mut CborDecoder::new(Cursor::new(buf)))?;
                   }

                _ => {bail!("Not a valid Tag!")}
            }

            next = cbor_decoder.peek();
        }
        Ok(())
    }

    fn parse_dict<R: Read + Seek>(&mut self, cbd: &mut CborDecoder<R>) {
        let size = cbd.read_map_length();

        for _i in 0..size {
            let idx = cbd.read_int() as usize;
            self.ftr.str_dict.insert(idx, cbd.read_text_string());
        }


    }

    fn parse_dir<R: Read + Seek>(&mut self, cbd: &mut CborDecoder<R>) -> color_eyre::Result<()>{
        let size = cbd.read_array_length();
        if size < 0 {
            let mut next_dir = cbd.peek();
            while next_dir.is_ok() && next_dir.unwrap() != 0xff {
                Self::parse_dir_entry(self, cbd)?;

                next_dir = cbd.peek();
            }

        }else {
            for _i in 1..size {
                Self::parse_dir_entry(self, cbd)?;
            }
        }
        Ok(())
    }


    fn parse_dir_entry<R: Read + Seek>(&mut self, cbd: &mut CborDecoder<R>) -> color_eyre::Result<()>{
        let dir_tag = cbd.read_tag();
        if dir_tag == STREAM as i64{
            let len = cbd.read_array_length();
            if len != 3 {
                bail!("Directory Entry(Stream) has wrong size!");
            }
            let stream_id = cbd.read_int() as usize;

            let name_id = cbd.read_int() as usize;
            let name = match self.ftr.str_dict.get(&name_id) {
                Some(n) => n,
                None => bail!("There is not entry in the Dictionary for id {name_id}"),
            };

            let kind_id = cbd.read_int() as usize;
            let kind = match self.ftr.str_dict.get(&kind_id) {
                Some(k) => k,
                None => bail!("There is not entry in the Dictionary for id {kind_id}"),
            };

            self.ftr.tx_streams.insert(stream_id, TxStream{id: stream_id, name: name.clone(), kind: kind.clone(), generators: vec![], tx_block_ids: vec![]});

        } else if dir_tag == GENERATOR as i64{
            let len = cbd.read_array_length();
            if len != 3 {
                bail!("Directory Entry(Generator) has wrong size!");
            }
            let gen_id = cbd.read_int() as usize;

            let name_id = cbd.read_int() as usize;

            let name = match self.ftr.str_dict.get(&name_id) {
                Some(n) => n,
                None => bail!("There is not entry in the Dictionary for id {name_id}"),
            };

            let stream_id = cbd.read_int() as usize;

            let generator = TxGenerator{id: gen_id, name: name.clone(), stream_id, transactions: vec![]};

            self.ftr.tx_generators.insert(gen_id, generator);
            self.ftr.tx_streams.get_mut(&stream_id).unwrap().generators.push(gen_id);
        }
        Ok(())
    }

    fn parse_tx_block<R: Read + Seek>(&mut self, cbd: &mut CborDecoder<R>) -> color_eyre::Result<()>{
        let size = cbd.read_array_length();
        if size != -1 {
            bail!("Transaction Block does not have indefinite length!");
        }

        let mut next_tx = cbd.peek();
        while next_tx.is_ok() && next_tx.unwrap() != 0xff {

            let arr_len = cbd.read_array_length();

            let mut event = Event::new();
            let mut attributes = vec![Attribute::new_empty(); 0];

            for _i in 0..arr_len {
                let tag = cbd.read_tag();

                match tag  as u64{
                    EVENT_TAG => {
                        let event_len = cbd.read_array_length();
                        if event_len != 4 {
                            bail!("Event has wrong size!");
                        }
                        let tx_id = cbd.read_int() as usize;
                        let gen_id = cbd.read_int() as usize;
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
                            bail!("Begin Attribute has wrong size!");
                        }

                        let new_begin = self.parse_attribute(cbd, BEGIN_TAG);
                        attributes.push(new_begin);
                    }
                    RECORD_TAG => {
                        let len = cbd.read_array_length();
                        if len != 3 {
                            bail!("Record Attribute has wrong size!");
                        }
                        let new_record = self.parse_attribute(cbd, RECORD_TAG);
                        attributes.push(new_record);
                    }
                    END_TAG => {
                        let len = cbd.read_array_length();
                        if len != 3 {
                            bail!("End Attribute has wrong size!");
                        }
                        let new_end = self.parse_attribute(cbd, END_TAG);
                        attributes.push(new_end);
                    }
                    _ => {bail!("Not a valid Transaction Block Tag")}
                }

            }

            let mut tx = Transaction{
                event,
                attributes,
                inc_relations: vec![],
                out_relations: vec![],
            };

            for rel in &self.ftr.tx_relations {
                if rel.source_tx_id == tx.event.tx_id {
                    tx.out_relations.push(rel.clone());
                } else if rel.sink_tx_id == tx.event.tx_id {
                    tx.inc_relations.push(rel.clone());
                }
            }

            self.ftr.tx_generators.get_mut(&tx.event.gen_id).unwrap().transactions.push(tx);

            next_tx = cbd.peek();

        }
        Ok(())
    }

    fn parse_rel<R: Read + Seek>(&mut self, cbd: &mut CborDecoder<R>) -> color_eyre::Result<()>{
        let size = cbd.read_array_length();
        if size != -1 {
            bail!("Relation block does not have indefinite size!");
        }

        let mut next_rel = cbd.peek();
        while next_rel.is_ok() && next_rel.unwrap() != 0xff {
            let sz = cbd.read_array_length();
            if sz != 5 && sz != 3 {
                bail!("Relation has wrong size");
            }
            let type_id = cbd.read_int() as usize;
            let from_tx_id = cbd.read_int() as usize;
            let to_tx_id = cbd.read_int() as usize;
            let from_stream_id = if sz > 3 {cbd.read_int() as usize} else {
                let mut stream_id: usize = 0;
                for curr_gen in &self.ftr.tx_generators {
                    let mut tx: Option<&Transaction> = None;
                    for curr_tx in &curr_gen.1.transactions {
                        if curr_tx.event.tx_id == from_tx_id {
                            tx = Some(curr_tx);
                            break;
                        }
                    }
                    if tx.is_some() && &tx.unwrap().event.gen_id == curr_gen.0 {
                        stream_id = curr_gen.1.stream_id;
                        break;
                    }
                }
                stream_id
            };
            let to_stream_id = if sz > 3 {cbd.read_int() as usize} else {
                let mut stream_id: usize = 0;
                for curr_gen in &self.ftr.tx_generators {
                    let mut tx: Option<&Transaction> = None;
                    for curr_tx in &curr_gen.1.transactions {
                        if curr_tx.event.tx_id == to_tx_id {
                            tx = Some(curr_tx);
                            break;
                        }
                    }
                    if tx.is_some() && &tx.unwrap().event.gen_id == curr_gen.0 {
                        stream_id = curr_gen.1.stream_id;
                        break;
                    }
                }
                stream_id

            };
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
        Ok(())
    }

    //loads the transactions of all generators of stream 'stream_id'
    pub(super) fn load_transactions(&mut self, stream_id: usize) -> color_eyre::Result<()>{
        if self.ftr.file_name == "" {
            bail!("Cannot load transaction when then input is not a file! \nTransactions should already be loaded.")
        }
        let reader = File::open(&self.ftr.file_name).unwrap();

        let tx_block_ids = self.ftr.tx_streams.get(&stream_id).unwrap().tx_block_ids.clone();

        for tx_block_id in tx_block_ids{

            let mut cbor_decoder = CborDecoder::new(&reader);

            cbor_decoder.input_stream.seek(SeekFrom::Start(tx_block_id.0)).expect("");

            if tx_block_id.1 {
                let uncomp_size = cbor_decoder.read_int();

                let mut buf = vec![0u8; uncomp_size as usize];
                let bytes = cbor_decoder.read_byte_string();
                match decompress_into(bytes.as_slice(), &mut buf) {
                    Ok(_) => {}
                    Err(e) => {bail!("Could not decompress compressed data correctly: {}", e)}
                }

                Self::parse_tx_block(self, &mut CborDecoder::new(Cursor::new(buf)))?;
            } else {
                Self::parse_tx_block(self, &mut CborDecoder::new(Cursor::new(cbor_decoder.read_byte_string())))?;
            }
        }
        Ok(())
    }

    fn parse_attribute<R: Read + Seek>(&self, cbd: &mut CborDecoder<R>, attribute_type: u64) -> Attribute{
        let name_id = cbd.read_int() as usize;
        let data_type = cbd.read_int();
        let data_type_with_value = match data_type as u8 {
            BOOLEAN => DataType::Boolean(cbd.read_boolean()),
            ENUMERATION => DataType::Enumeration(self.ftr.str_dict.get(&(cbd.read_int() as usize)).unwrap().clone()),
            INTEGER => DataType::Integer(cbd.read_int() as u64),
            UNSIGNED => DataType::Unsigned(cbd.read_int()),
            FLOATING_POINT_NUMBER => DataType::FloatingPointNumber(cbd.read_float()),
            BIT_VECTOR => DataType::BitVector(self.ftr.str_dict.get(&(cbd.read_int() as usize)).unwrap().clone()),
            LOGIC_VECTOR => DataType::LogicVector(self.ftr.str_dict.get(&(cbd.read_int() as usize)).unwrap().clone()),
            FIXED_POINT_INTEGER => DataType::FixedPointInteger(cbd.read_float()),
            UNSIGNED_FIXED_POINT_INTEGER => DataType::UnsignedFixedPointInteger(cbd.read_float()),
            POINTER => DataType::Pointer(cbd.read_int() as u64),
            STRING => DataType::String(self.ftr.str_dict.get(&(cbd.read_int() as usize)).unwrap().clone()),
            TIME => DataType::Time(cbd.read_int() as u64),
            _ => DataType::Error,
        };

        let kind = match attribute_type {
            BEGIN_TAG => AttributeType::BEGIN,
            RECORD_TAG => AttributeType::RECORD,
            END_TAG => AttributeType::END,
            _ => AttributeType::NONE,
        };

        Attribute{
            kind,
            name: self.ftr.str_dict.get(&name_id).unwrap().clone(),
            data_type: data_type_with_value,
        }
    }
}