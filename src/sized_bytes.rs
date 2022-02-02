use hex::{decode, encode};
use std::error::Error;
use std::str::Bytes;

pub trait SizedBytes {
    fn from_bytes(bytes: Bytes) -> Self;
    fn from_hexstr(hex: String) -> Self;
    fn to_hex(&self) -> String;
    fn to_bytes(&self) -> Bytes;
    fn to_string(&self) -> String {
        self.to_hex()
    }
}

pub fn prep_hex_str(to_fix: &String) -> String {
    if to_fix.starts_with("0x") {
        to_fix.strip_prefix("0x").unwrap().to_string()
    } else {
        to_fix.to_string()
    }
}

pub fn u64_to_bytes(v: u64) -> Vec<u8> {
    let mut rtn = Vec::new();
    if v.leading_zeros() == 0 {
        rtn.push(u8::MIN);
        let ary = v.to_be_bytes();
        rtn.extend_from_slice(&ary);
        rtn
    } else {
        let mut trim: bool = true;
        for b in v.to_be_bytes() {
            if trim {
                if b == u8::MIN {
                    continue;
                } else {
                    rtn.push(b);
                    trim = false;
                }
            } else {
                rtn.push(b);
            }
        }
        rtn
    }
}

pub struct Bytes4<'a> {
    size: u8,
    _bytes: Bytes<'a>,
}
impl SizedBytes for Bytes4<'_> {
    fn from_bytes(bytes: Bytes) -> Result<Self, Err> {
        if bytes.len() != 4 {
            Err(format!("Invalid Byte Length for bytes4: {}", bytes.len()).into())
        } else {
            Ok(Bytes4 {
                size: 4,
                _bytes: bytes,
            })
        }
    }

    fn from_hexstr(hex: &String) -> Result<Self, FromHexError> {
        let bytes: Vec<u8> = decode(prep_hex_str(hex)).unwrap();
        if bytes.len() != 4 {
            Err(format!("Invalid Byte Length for bytes4: {}", bytes.len()).into())
        } else {
            Ok(Bytes4 {
                size: 4,
                _bytes: Bytes::from(bytes),
            })
        }
    }

    fn to_hex(&self) -> String {
        encode(self._bytes)
    }

    fn to_bytes(&self) -> Bytes {
        self._bytes.clone()
    }
}
