use std::collections::HashMap;
use std::fmt::Debug;
use serde::{Deserialize, Serialize};
use crate::types::DataType::NONE;
use crate::types::Timescale::{Fs, Ms, Ns, Ps, S, Unit, Us};

#[derive(Debug, Serialize, Deserialize)]
pub struct TxStream {
    pub id: i64,
    pub name: String,
    pub kind: String,
    pub generators: Vec<TxGenerator>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TxGenerator {
    pub id: i64,
    pub name: String,
    pub stream_id: i64, // TODO make reference to stream not id of stream
    pub transactions: Vec<Transaction>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(super) struct TxRelation {
    pub name: String,
    pub source_tx_id: i64,
    pub sink_tx_id: i64,
    pub source_stream_id: i64,
    pub sink_stream_id: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub(super) struct TxBlock {
    pub stream_id: i64,
    pub start_time: i64,
    pub end_time: i64,
    pub transactions: Vec<Transaction>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Transaction {
    pub event: Event,
    pub attributes: Vec<Attribute>,
    pub relations: Vec<TxRelation>,
}

impl Transaction {
    pub fn new() -> Self{
        let event = Event::new();
        let attributes = vec![Attribute::new(); 0];
        let relations: Vec<TxRelation> = vec![];

        Self {
            event,
            attributes,
            relations,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Event {
    pub tx_id: i64,
    pub gen_id: i64,
    pub start_time: i64,
    pub end_time: i64,
}

impl Event {
    pub fn new() -> Self{
        let tx_id = -1;
        let gen_id = -1;
        let start_time = -1;
        let end_time = -1;
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
    pub data_type: DataType, // TODO make it so enum carries the value of the respective data_type
    pub value: i64,
}

impl Attribute {
    pub fn new() -> Self{
        let kind = AttributeType::NONE;
        let name = String::new();
        let data_type = NONE;
        let value = -1;
        Self {
            kind,
            name,
            data_type,
            value
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) enum DataType {
    BOOLEAN,
    ENUMERATION,
    INTEGER,
    UNSIGNED,
    FLOATING_POINT_NUMBER,
    BIT_VECTOR,
    LOGIC_VECTOR,
    FIXED_POINT_INTERGER,
    UNSIGNED_FIXED_POINT_INTEGER,
    POINTER,
    STRING,
    TIME,
    NONE,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) enum AttributeType {
    BEGIN,
    RECORD,
    END,
    NONE,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FTR {
    pub time_scale: Timescale,
    pub str_dict: HashMap<i64, String>,
    pub tx_streams: Vec<TxStream>,
    //pub tx_generators: Vec<TxGenerator>, // TODO REMOVE
    //pub tx_blocks: Vec<TxBlock>,    // TODO REMOVE
    // pub tx_relations: Vec<TxRelation>, // TODO REMOVE
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
            _ => Unit,
        }
    }
}