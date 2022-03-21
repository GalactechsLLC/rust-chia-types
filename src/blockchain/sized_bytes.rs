use hex::FromHexError;
use hex::{decode, encode};
use serde::de::Visitor;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::error::Error;
use std::fmt;

fn prep_hex_str(to_fix: &String) -> String {
    let lc = to_fix.to_lowercase();
    if lc.starts_with("0x") {
        lc.strip_prefix("0x").unwrap().to_string()
    } else {
        lc.to_string()
    }
}

pub fn hex_to_bytes(hex: &String) -> Result<Vec<u8>, FromHexError> {
    decode(prep_hex_str(hex))
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

pub trait SizedBytes<'a>: Serialize + Deserialize<'a> + fmt::Display {
    const SIZE: usize;
    fn new(bytes: Vec<u8>) -> Self;
    fn to_bytes(&self) -> Vec<u8>;
}

macro_rules! impl_sized_bytes {

    ($($name: ident, $size:expr, $visitor:ident);*) => {
        $(
            #[derive(Debug, PartialEq, Eq, Clone, Hash)]
            pub struct $name {
                bytes: Vec<u8>,
            }
            impl<'a> SizedBytes<'a> for $name {
                const SIZE: usize = $size;

                fn new(bytes: Vec<u8>) -> Self {
                    $name { bytes: bytes }
                }

                fn to_bytes(&self) -> Vec<u8> {
                    self.bytes.clone()
                }
            }
            impl Serialize for $name {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: Serializer,
                {
                    serializer.serialize_str(self.to_string().as_str())
                }
            }
            impl fmt::Display for $name {
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    write!(f, "{}", encode(&self.bytes))
                }
            }

            impl From<Vec<u8>> for $name {
                fn from(bytes: Vec<u8>) -> Self {
                    if 0 != Self::SIZE && bytes.len() > Self::SIZE {
                        $name { bytes : bytes[..Self::SIZE].to_vec() }
                    } else if 0 != Self::SIZE && bytes.len() < Self::SIZE {
                        let mut m_bytes: Vec<u8> = Vec::new();
                        m_bytes.extend(&bytes);
                        m_bytes.append(&mut b"\x00".repeat(Self::SIZE));
                        $name { bytes : m_bytes[..Self::SIZE].to_vec() }
                    } else {
                        $name { bytes : bytes }
                    }
                }
            }

            impl Into<Vec<u8>> for $name {
                fn into(self) -> Vec<u8> {
                    self.bytes.clone()
                }
            }

            impl From<String> for $name {
                fn from(hex: String) -> Self {
                    let bytes: Vec<u8> = decode(prep_hex_str(&hex)).unwrap();
                    if 0 != Self::SIZE && bytes.len() > Self::SIZE {
                        $name { bytes : bytes[..Self::SIZE].to_vec() }
                    } else if 0 != Self::SIZE && bytes.len() < Self::SIZE {
                        let mut m_bytes: Vec<u8> = Vec::new();
                        m_bytes.extend(&bytes);
                        m_bytes.append(&mut b"\x00".repeat(Self::SIZE));
                        $name { bytes : m_bytes[..Self::SIZE].to_vec() }
                    } else {
                        $name { bytes : bytes }
                    }
                }
            }

            impl From<&str> for $name {
                fn from(hex: &str) -> Self {
                    let bytes: Vec<u8> = decode(prep_hex_str(&hex.to_string())).unwrap();
                    if 0 != Self::SIZE && bytes.len() > Self::SIZE {
                        $name { bytes : bytes[..Self::SIZE].to_vec() }
                    } else if 0 != Self::SIZE && bytes.len() < Self::SIZE {
                        let mut m_bytes: Vec<u8> = Vec::new();
                        m_bytes.extend(&bytes);
                        m_bytes.append(&mut b"\x00".repeat(Self::SIZE));
                        $name { bytes : m_bytes[..Self::SIZE].to_vec() }
                    } else {
                        $name { bytes : bytes }
                    }
                }
            }

            struct $visitor;

            impl<'de> Visitor<'de> for $visitor {
                type Value = $name;

                fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                    formatter.write_str(format!("Expecting a hex String, or byte array of size{}", $size).as_str())
                }

                fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
                where
                    E: Error,
                {
                    Ok(value.into())
                }

            }

            impl<'a> Deserialize<'a> for $name {
                fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where
                    D: Deserializer<'a>,
                {
                    match deserializer.deserialize_string($visitor) {
                        Ok(hex) => Ok(hex),
                        Err(er) => Err(er),
                    }
                }
            }
        )*
    };
    ()=>{};
}

impl_sized_bytes!(
    UnsizedBytes, 0, UnsizedBytesVisitor;
    Bytes4, 4, Bytes4Visitor;
    Bytes8, 8, Bytes8Visitor;
    Bytes16, 16, Bytes16Visitor;
    Bytes32, 32, Bytes32Visitor;
    Bytes48, 48, Bytes48Visitor;
    Bytes96, 96, Bytes96Visitor;
    Bytes192, 192, Bytes192Visitor
);
