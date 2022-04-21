use crate::blockchain::sized_bytes::hex_to_bytes;
use crate::clvm::program::Program;
use crate::clvm::utils::MEMPOOL_MODE;
use clvmr::allocator::{Allocator, NodePtr};
use clvmr::chia_dialect::ChiaDialect;
use clvmr::cost::Cost;
use clvmr::run_program::run_program;
use clvmr::serialize::node_from_bytes;
use hex::encode;
use serde::de::Visitor;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::error::Error;
use std::fmt;
use std::fs;
use std::path::Path;

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct SerializedProgram {
    buffer: Vec<u8>,
}
impl SerializedProgram {
    pub fn from_file(path: &Path) -> SerializedProgram {
        SerializedProgram {
            buffer: fs::read(path).unwrap(),
        }
    }
    pub fn from_bytes(bytes: &Vec<u8>) -> SerializedProgram {
        SerializedProgram {
            buffer: bytes.clone(),
        }
    }
    pub fn from_hex(hex_str: String) -> SerializedProgram {
        SerializedProgram {
            buffer: hex_to_bytes(&hex_str).unwrap_or(Vec::new()),
        }
    }
    //pub fn uncurry(&self) -> (SerializedProgram, SerializedProgram) {}
    pub fn to_string(&self) -> String {
        encode(&self.buffer)
    }
    pub fn to_bytes(&self) -> Vec<u8> {
        self.buffer.clone()
    }

    pub fn run_mempool_with_cost(
        &self,
        allocator: &mut Allocator,
        max_cost: Cost,
        args: &Program,
    ) -> Result<(u64, NodePtr), Box<dyn Error>> {
        self.run(allocator, max_cost, MEMPOOL_MODE, args)
    }

    pub fn run_with_cost(
        &self,
        allocator: &mut Allocator,
        max_cost: Cost,
        args: &Program,
    ) -> Result<(u64, NodePtr), Box<dyn Error>> {
        self.run(allocator, max_cost, 0, args)
    }

    pub fn to_program<'a>(self) -> Result<Program, Box<dyn Error>> {
        Ok(Program::new(self.buffer.clone()))
    }

    fn run(
        &self,
        allocator: &mut Allocator,
        max_cost: Cost,
        flags: u32,
        args: &Program,
    ) -> Result<(u64, NodePtr), Box<dyn Error>> {
        let program = node_from_bytes(allocator, &self.buffer.as_slice())?;
        let args = node_from_bytes(allocator, args.serialized.as_slice())?;
        let dialect = ChiaDialect::new(flags);
        match run_program(allocator, &dialect, program, args, max_cost, None) {
            Ok(reduct) => Ok((reduct.0, reduct.1)),
            Err(error) => Err(error.1.into()),
        }
    }
}
impl From<String> for SerializedProgram {
    fn from(hex: String) -> Self {
        SerializedProgram::from_hex(hex)
    }
}

impl From<&str> for SerializedProgram {
    fn from(hex: &str) -> Self {
        SerializedProgram::from_hex(hex.to_string())
    }
}
struct SerializedProgramVisitor;

impl<'de> Visitor<'de> for SerializedProgramVisitor {
    type Value = SerializedProgram;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str(format!("Expecting a hex String, or byte array").as_str())
    }

    fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(value.into())
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(value.into())
    }
}

impl Serialize for SerializedProgram {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.to_string().as_str())
    }
}

impl<'a> Deserialize<'a> for SerializedProgram {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'a>,
    {
        match deserializer.deserialize_string(SerializedProgramVisitor) {
            Ok(hex) => Ok(hex),
            Err(er) => Err(er),
        }
    }
}
