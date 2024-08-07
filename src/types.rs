use std::collections::HashMap;
use std::fmt::{Debug};
use num_bigint::{BigInt, BigUint};
use serde::{Deserialize, Serialize};
use crate::types::DataType::Error;
use crate::types::Timescale::{Fs, Ms, Ns, Ps, S, Us};
use core::fmt;
use std::path::PathBuf;
use crate::ftr_parser::FtrParser;

type IsCompressed = bool;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TxStream {
    pub id: usize,
    pub name: String,
    pub kind: String,
    pub generators: Vec<usize>,
    pub transactions_loaded: bool,
    pub(super) tx_block_ids: Vec<(u64, IsCompressed)>,
}

impl PartialEq<Self> for TxStream {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TxGenerator {
    pub id: usize,
    pub stream_id: usize,
    pub name: String,
    pub transactions: Vec<Transaction>,
}

impl PartialEq<Self> for TxGenerator {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id &&
            self.stream_id == other.stream_id
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TxRelation {
    pub name: String,
    pub source_tx_id: usize,
    pub sink_tx_id: usize,
    pub source_stream_id: usize,
    pub sink_stream_id: usize
}

impl PartialEq<Self> for TxRelation {
    fn eq(&self, other: &Self) -> bool {
        self.source_tx_id == other.source_tx_id &&
            self.sink_tx_id == other.sink_tx_id &&
            self.source_stream_id == other.source_stream_id &&
            self.sink_stream_id == other.sink_stream_id
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Transaction {
    pub event: Event,
    pub attributes: Vec<Attribute>,
    pub inc_relations: Vec<TxRelation>,
    pub out_relations: Vec<TxRelation>,
}

impl PartialEq<Self> for Transaction {
    fn eq(&self, other: &Self) -> bool {
        self.event.tx_id == other.event.tx_id &&
            self.event.gen_id == other.event.gen_id
    }
}

impl Transaction {
    pub fn get_tx_id(&self) -> usize {
        self.event.tx_id
    }

    pub fn get_gen_id(&self) -> usize {
        self.event.gen_id
    }

    pub fn get_start_time(&self) -> BigUint {
        self.event.start_time.clone()
    }

    pub fn get_end_time(&self) -> BigUint {
        self.event.end_time.clone()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Event {
    pub tx_id: usize,
    pub gen_id: usize,
    pub start_time: BigUint,
    pub end_time: BigUint,
}

impl Event {
    pub fn new() -> Self{
        let tx_id = 0;
        let gen_id = 0;
        let start_time = BigUint::default();
        let end_time = BigUint::default();
        Self {
            tx_id,
            gen_id,
            start_time,
            end_time
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attribute {
    pub kind: AttributeType,
    pub name: String,
    pub data_type: DataType,
}

impl Attribute {
    pub fn new_empty() -> Self{
        let kind = AttributeType::NONE;
        let name = String::new();
        let data_type = Error;
        Self {
            kind,
            name,
            data_type,
        }
    }

    pub fn new_begin(name: String, data_type: DataType) -> Self {
        Self {
            kind: AttributeType::BEGIN,
            name,
            data_type
        }
    }

    pub fn new_record(name: String, data_type: DataType) -> Self {
        Self {
            kind: AttributeType::RECORD,
            name,
            data_type
        }
    }

    pub fn new_end(name: String, data_type: DataType) -> Self {
        Self {
            kind: AttributeType::END,
            name,
            data_type
        }
    }

    pub fn value(&self) -> String {
        match &self.data_type {
            DataType::Boolean(b) => b.to_string(),
            DataType::Enumeration(s) => s.clone(),
            DataType::Integer(i) => i.to_string(),
            DataType::Unsigned(u) => u.to_string(),
            DataType::FloatingPointNumber(f) => f.to_string(),
            DataType::BitVector(s) => s.clone(),
            DataType::LogicVector(s) => s.clone(),
            DataType::FixedPointInteger(f) => f.to_string(),
            DataType::UnsignedFixedPointInteger(f) => f.to_string(),
            DataType::Pointer(u) => u.to_string(),
            DataType::String(s) => s.clone(),
            DataType::Time(u) => u.to_string(),
            Error => "".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataType {
    Boolean(bool),
    Enumeration(String),
    Integer(i64),
    Unsigned(u64),
    FloatingPointNumber(f32),
    BitVector(String),
    LogicVector(String),
    FixedPointInteger(f32),
    UnsignedFixedPointInteger(f32),
    Pointer(u64),
    String(String),
    Time(u64),
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AttributeType {
    BEGIN,
    RECORD,
    END,
    NONE,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FTR {
    pub time_scale: Timescale,
    pub max_timestamp: BigInt,
    pub str_dict: HashMap<usize, String>,
    pub tx_streams: HashMap<usize, TxStream>,
    pub tx_generators: HashMap<usize, TxGenerator>,
    pub tx_relations: Vec<TxRelation>,
    pub(crate) path: Option<PathBuf>,
}

impl FTR {
    // Takes a stream id and loads all associated transactions into memory
    pub fn load_stream_into_memory(&mut self, stream_id: usize) -> color_eyre::Result<()>{
        let mut ftr_parser = FtrParser::new(self);

        ftr_parser.load_transactions(stream_id)?;

        Ok(())
    }

    // drops all transactions from this stream from memory, but the stream itself doesn't get deleted
    pub fn drop_stream_from_memory(&mut self, stream_id: usize) {
        for gen_id in &self.tx_streams.get(&stream_id).expect("").generators {
            self.tx_generators.get_mut(gen_id).unwrap().transactions = vec![];
        }
    }

    pub fn get_stream(&self, stream_id: usize) -> Option<&TxStream> {
        self.tx_streams.get(&stream_id)
    }

    pub fn get_stream_from_name(&self, name: String) -> Option<&TxStream> {
        self.tx_streams
            .values()
            .find(|t| t.name == name)
    }

    pub fn get_generator(&self, gen_id: usize) -> Option<&TxGenerator> {
        self.tx_generators.get(&gen_id)
    }

    /// Returns the `Optional<TxGenerator>` with the name `gen_name` from the stream with id `stream_id`.
    pub fn get_generator_from_name(&self, stream_id: usize, gen_name: String) -> Option<&TxGenerator> {
        self.tx_streams
            .get(&stream_id)
            .unwrap()
            .generators
            .iter()
            .map(|id| self.tx_generators.get(id).unwrap())
            .find(|gen| gen.name == gen_name)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
pub enum Timescale {
    Fs,
    Ps,
    Ns,
    Us,
    Ms,
    S,
    Unit,
    None,
}

impl Timescale {
    pub fn get_timescale(exponent: i64) -> Timescale{
        match exponent {
            0 => S,
            -4 => Ms,
            -8 => Us,
            -12 => Ns,
            -16 => Ps,
            -20 => Fs,
            _ => Timescale::None,
        }
    }
}

impl fmt::Display for Timescale {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}