use hex::FromHexError;
use hex::{decode, encode};
use serde::de::Visitor;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
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
    fn from_bytes(bytes: Vec<u8>) -> Self;
    fn from_hexstr(hex: String) -> Self;
    fn to_bytes(&self) -> Vec<u8>;
    fn to_hex(&self) -> String;
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

                fn from_bytes(bytes: Vec<u8>) -> Self {
                    if 0 != Self::SIZE && bytes.len() != Self::SIZE {
                        panic!(
                            "Invalid Byte Length for bytes{}: {}",
                            Self::SIZE,
                            bytes.len()
                        )
                    } else {
                        $name { bytes : bytes }
                    }
                }

                fn from_hexstr(hex: String) -> Self {
                    let bytes: Vec<u8> = decode(prep_hex_str(&hex)).unwrap();
                    if 0 != Self::SIZE && bytes.len() != Self::SIZE {
                        panic!(
                            "Invalid Byte Length for bytes{}: {}",
                            Self::SIZE,
                            bytes.len()
                        )
                    }
                    $name { bytes : bytes }
                }
                fn to_hex(&self) -> String {
                    encode(&self.to_bytes())
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
                    write!(f, "{}", self.to_hex())
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
                    E: serde::de::Error,
                {
                    Ok($name::from_hexstr(value))
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
