use std::io::{Error, Read, Seek, SeekFrom};
use half::f16;

const ONE_BYTE: u8 = 24;
const TWO_BYTES: u8 = 25;
const FOUR_BYTES: u8 = 26;
const EIGHT_BYTES: u8 = 27;
const BREAK: u8 = 31;

const TYPE_UNSIGNED_INT: u8 = 0x00;
const TYPE_NEGATIVE_INT: u8 = 0x01;
const TYPE_BYTE_STRING: u8 = 0x02;
const TYPE_TEXT_STRING: u8 = 0x03;
const TYPE_ARRAY: u8 = 0x04;
const TYPE_MAP: u8 = 0x5;
const TYPE_TAG: u8 = 0x06;
const TYPE_FLOAT_SIMPLE: u8 = 0x07;

const FALSE: u8 = 0x14;
const TRUE: u8 = 0x15;
const HALF_PRECISION_FLOAT: u8 = 0x19;
const SINGLE_PRECISION_FLOAT: u8 = 0x1a;
const DOUBLE_PRECISION_FLOAT: u8 = 0x1b;

pub struct CborDecoder<R>{
    pub(crate) input_stream: R,
    peek_buf: Vec<u8>,
}

impl <R: Read + Seek>CborDecoder<R>{
    pub fn new(input_stream: R) -> Self {
        let mut peek_buf = vec![0u8; 1];
        peek_buf.clear();
        Self {input_stream, peek_buf}
    }

    pub fn read_tag(&mut self) -> i64 {
        let length = Self::read_major_type(self, TYPE_TAG).unwrap();
        Self::read_unsigned_int(self, length , false)
    }


    pub fn read_major_type(&mut self, major_type: u8) -> Result<u8, Error> {
        let mut buf = vec![0u8; 1];
        if !self.peek_buf.is_empty() {
            buf[0] = self.peek_buf[0];
            self.peek_buf.clear();
        } else {
            self.input_stream.read_exact(&mut buf)?
        };

        if major_type != ((buf[0] >> 5) & 0x07) {
            panic!()
        }
        Ok(buf[0] & 0x1F)
    }

    pub fn read_major_type_with_size(&mut self, major_type: u8) -> i64{
        let length = Self::read_major_type(self, major_type).unwrap();
        Self::read_unsigned_int(self, length, true)
    }

    pub fn read_major_type_exact(&mut self, major_type: u8, sub_type: u8) {
        let sub_t = self.read_major_type(major_type).expect("Could not read major type!");
        if (sub_t ^ sub_type) != 0 {
            panic!("Expected and actual subtype do not match up!")
        }
    }

    pub fn read_array_length(&mut self) -> i64 {
        Self::read_major_type_with_size(self, TYPE_ARRAY)
    }

    pub fn read_unsigned_int(&mut self, length: u8, break_allowed: bool) -> i64{
        let mut result = -1;
        if length < 24 {
            result = length as i64;
        } else if length == ONE_BYTE {
            result = Self::read_unsigned_int_8(self);
        } else if length == TWO_BYTES {
            result = Self::read_unsigned_int_16(self);
        } else if length == FOUR_BYTES {
            result = Self::read_unsigned_int_32(self);
        } else if length == EIGHT_BYTES {
            result = Self::read_unsigned_int_64(self);
        } else if break_allowed && length == BREAK {
            return -1
        }
        result
    }

    fn read_unsigned_int_8(&mut self) -> i64{
        let mut buf = vec![0u8; 1];
        self.input_stream.read_exact(&mut buf).expect("");
        buf[0] as i64
    }

    fn read_unsigned_int_16(&mut self) -> i64 {
        let mut buf = vec![0u8; 2];
        self.input_stream.read_exact(&mut buf).expect("");
        (buf[0] as i64) << 8 | (buf[1] as i64)
    }

    fn read_unsigned_int_32(&mut self) -> i64 {
        let mut buf = vec![0u8; 4];
        self.input_stream.read_exact(&mut buf).expect("");
        (buf[0] as i64) << 24 | (buf[1] as i64) << 16 | (buf[2] as i64) << 8 | (buf[3] as i64)
    }

    fn read_unsigned_int_64(&mut self) -> i64 {
        let mut buf = vec![0u8; 8];
        self.input_stream.read_exact(&mut buf).expect("");
        (buf[0] as i64) << 56 | (buf[1] as i64) << 48 | (buf[2] as i64) << 40 | (buf[3] as i64) << 32 | //
            (buf[4] as i64) << 24 | (buf[5] as i64) << 16 | (buf[6] as i64) << 8 | (buf[7] as i64)
    }

    pub fn read_boolean(&mut self) -> bool {
        let b = self.read_major_type(TYPE_FLOAT_SIMPLE).expect("Not a boolean value!");
        b == TRUE
    }

    pub fn read_double(&mut self) -> f64 {
        self.read_major_type_exact(TYPE_FLOAT_SIMPLE, DOUBLE_PRECISION_FLOAT);

        f64::from_be_bytes(self.read_unsigned_int_64().to_be_bytes())
    }

    pub fn read_float(&mut self) -> f32 {
        self.read_major_type_exact(TYPE_FLOAT_SIMPLE, SINGLE_PRECISION_FLOAT);

        f32::from_be_bytes((self.read_unsigned_int_32() as u32).to_be_bytes())
    }

    pub fn read_half_precision_float(&mut self) -> f16 {
        self.read_major_type_exact(TYPE_FLOAT_SIMPLE, HALF_PRECISION_FLOAT);

        f16::from_be_bytes((self.read_unsigned_int_16() as u16).to_be_bytes())
    }


    pub fn read_byte_string(&mut self) -> Vec<u8>{
        let len = Self::read_major_type_with_size(self, TYPE_BYTE_STRING);
        let mut buf = vec![0u8; len as usize];
        self.input_stream.read_exact(&mut buf).expect("");
        buf
    }

    pub fn skip_byte_string(&mut self) {
        let len = Self::read_major_type_with_size(self, TYPE_BYTE_STRING);
        self.input_stream.seek(SeekFrom::Current(len)).expect("");
    }

    pub fn read_int(&mut self) -> i64 {
        let mut buf = vec![0u8; 1];
        self.input_stream.read_exact(&mut buf).expect("");

        let ui = Self::expect_integer_type(self, buf[0]);

        ui ^ Self::read_unsigned_int(self, buf[0] & 0x1f, false)
    }

    pub fn expect_integer_type(&mut self, ib: u8) -> i64 {
        let major_type = (ib & 0xff) >> 5;
        if (major_type != TYPE_UNSIGNED_INT) && (major_type != TYPE_NEGATIVE_INT) {
            panic!()
        }else {
            return -(major_type as i64)
        }
    }

    pub fn read_map_length(&mut self) -> i64{
        Self::read_major_type_with_size(self, TYPE_MAP)
    }

    pub fn read_text_string(&mut self) -> String{
        let len = Self::read_major_type_with_size(self, TYPE_TEXT_STRING);
        if len < 0 {
            panic!("Infinite length text are not supported!");
        }
        if len > i64::MAX {
            panic!("String length too long!");
        }
        let mut buf = vec![0u8; len as usize];
        self.input_stream.read_exact(&mut buf).expect("");
        String::from_utf8(buf).expect("")
    }

    pub fn peek(&mut self)  -> Result<i64, Error>{
        self.peek_buf = vec![0u8; 1];
        if let Err(err) = self.input_stream.read_exact(&mut self.peek_buf) {
            return Err(err)
        }
        Ok(self.peek_buf[0] as i64)
    }
}

