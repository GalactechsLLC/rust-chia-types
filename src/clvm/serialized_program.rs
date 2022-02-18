use crate::blockchain::sized_bytes::{hex_to_bytes, Bytes32};
use crate::clvm::program::Program;
use crate::clvm::utils::tree_hash;
use crate::clvm::utils::MEMPOOL_MODE;
use clvmr::allocator::{Allocator, NodePtr};
use clvmr::chia_dialect::ChiaDialect;
use clvmr::cost::Cost;
use clvmr::run_program::run_program;
use clvmr::serialize::node_from_bytes;
use hex::encode;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::error::Error;
use std::fs;
use std::path::Path;

#[derive(Clone, Eq, PartialEq, Serialize, Deserialize, Debug)]
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
    pub fn get_tree_hash(&self, args: &Vec<Bytes32>) -> Result<Bytes32, Box<dyn Error>> {
        let mut alloc = Allocator::new();
        match node_from_bytes(&mut alloc, &self.buffer.as_slice()) {
            Ok(node_ptr) => tree_hash(&alloc, node_ptr, &HashSet::from_iter(args.clone())),
            Err(error) => Err(Box::new(error)),
        }
    }

    pub fn run_mempool_with_cost(
        &self,
        allocator: &mut Allocator,
        max_cost: Cost,
        args: &[u8],
    ) -> Result<(u64, NodePtr), Box<dyn Error>> {
        self.run(allocator, max_cost, MEMPOOL_MODE, args)
    }
    pub fn run_with_cost(
        &self,
        allocator: &mut Allocator,
        max_cost: Cost,
        args: Vec<u8>,
    ) -> Result<(u64, NodePtr), Box<dyn Error>> {
        self.run(allocator, max_cost, 0, args.as_slice())
    }

    pub(crate) fn to_program(&self) -> Result<Program, Box<dyn Error>> {
        Program::new(self.buffer.clone())
    }

    fn run(
        &self,
        allocator: &mut Allocator,
        max_cost: Cost,
        flags: u32,
        args: &[u8],
    ) -> Result<(u64, NodePtr), Box<dyn Error>> {
        let program = node_from_bytes(allocator, &self.buffer.as_slice())?;
        let args = node_from_bytes(allocator, args)?;
        let dialect = ChiaDialect::new(flags);
        match run_program(allocator, &dialect, program, args, max_cost, None) {
            Ok(reduct) => Ok((reduct.0, reduct.1)),
            Err(error) => Err(error.1.into()),
        }
    }
}
