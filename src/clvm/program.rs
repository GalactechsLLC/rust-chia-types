use crate::blockchain::sized_bytes::*;
use crate::clvm::curry_utils::{curry, uncurry};
use crate::clvm::serialized_program::SerializedProgram;
use clvmr::allocator::SExp::{Atom, Pair};
use clvmr::allocator::{Allocator, SExp};
use clvmr::node::Node;
use clvmr::serialize::{node_from_bytes, node_to_bytes};
use num_bigint::BigInt;
use serde::ser::StdError;
use std::collections::HashMap;
use std::error::Error;
use std::str;

#[derive(Clone)]
pub struct Program {
    pub serialized: Vec<u8>,
}
impl Program {
    pub fn curry(&self, args: &[u8]) -> Result<Program, Box<dyn Error>> {
        let (_cost, program) = curry(&SerializedProgram::from_bytes(&self.serialized), args)?;
        Ok(program)
    }

    pub fn uncurry(&self) -> Result<(Program, Program), Box<dyn Error>> {
        let serial_program = SerializedProgram::from_bytes(&self.serialized);
        match uncurry(&serial_program)? {
            Some((program, args)) => Ok((program.to_program()?, args.to_program()?)),
            None => Ok((serial_program.to_program()?, 0.into())),
        }
    }

    fn null_pointer(&self) -> Result<bool, Box<dyn Error>> {
        let mut alloc = Allocator::new();
        match self.to_sexp(&mut alloc)? {
            Pair(_, _) => Ok(false),
            Atom(atom_buf) => Ok(alloc.buf(&atom_buf).len() == 0),
        }
    }

    pub fn as_atom_list(&self) -> Option<Vec<Vec<u8>>> {
        let mut rtn: Vec<Vec<u8>> = Vec::new();
        let mut alloc = Allocator::new();
        match node_from_bytes(&mut alloc, &self.serialized) {
            Ok(program) => {
                let mut cur_node = program;
                loop {
                    let sexp = alloc.sexp(cur_node);
                    match sexp {
                        Atom(_buf) => break,
                        Pair(node_ptr, node_ptr2) => {
                            rtn.extend([Vec::from(alloc.atom(node_ptr))]);
                            cur_node = node_ptr2;
                        }
                    }
                }
                Some(rtn)
            }
            Err(_error) => None,
        }
    }

    pub fn to_map(&self) -> Result<HashMap<String, Vec<u8>>, Box<dyn Error>> {
        let mut rtn: HashMap<String, Vec<u8>> = HashMap::new();
        let mut alloc = Allocator::new();
        let mut sexp = self.to_sexp(&mut alloc)?;
        loop {
            match sexp {
                Atom(_) => break,
                Pair(ptr, ptr2) => {
                    sexp = alloc.sexp(ptr2);
                    match alloc.sexp(ptr) {
                        Atom(buf) => {
                            rtn.insert(str::from_utf8(alloc.buf(&buf))?.to_string(), Vec::new());
                        }
                        Pair(key, value) => {
                            rtn.insert(
                                str::from_utf8(alloc.atom(key))?.to_string(),
                                alloc.atom(value).to_vec(),
                            );
                        }
                    }
                }
            }
        }
        Ok(rtn)
    }

    pub fn get_tree_hash(&self, args: &Vec<Bytes32>) -> Result<Bytes32, Box<dyn Error>> {
        SerializedProgram::from_bytes(&self.serialized).get_tree_hash(args)
    }

    pub fn to_sexp(&self, alloc: &mut Allocator) -> Result<SExp, Box<dyn Error>> {
        let node = node_from_bytes(alloc, &self.serialized)?;
        Ok(alloc.sexp(node))
    }

    pub fn is_atom(&self) -> Result<bool, Box<dyn Error>> {
        let mut alloc = Allocator::new();
        match self.to_sexp(&mut alloc)? {
            Pair(_, _) => Ok(false),
            Atom(_) => Ok(true),
        }
    }

    pub fn as_atom(&self) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut alloc = Allocator::new();
        match self.to_sexp(&mut alloc)? {
            Pair(_, _) => Err("Not an Atom".into()),
            Atom(atom_buf) => Ok(alloc.buf(&atom_buf).to_vec()),
        }
    }

    pub fn as_int(&self) -> Result<BigInt, Box<dyn Error>> {
        Ok(BigInt::from_signed_bytes_be(self.as_atom()?.as_slice()))
    }

    pub fn first(&self) -> Result<Program, Box<dyn Error>> {
        let mut alloc = Allocator::new();
        match self.to_sexp(&mut alloc)? {
            Pair(node_ptr, _node_ptr2) => {
                let node = Node::new(&alloc, node_ptr);
                match node_to_bytes(&node) {
                    Ok(serial_data) => Ok(Program::new(serial_data)),
                    Err(error) => Err(Box::new(error)),
                }
            }
            Atom(_buf) => Err("First of non-cons".into()),
        }
    }

    pub fn rest(&self) -> Result<Program, Box<dyn Error>> {
        let mut alloc = Allocator::new();
        match self.to_sexp(&mut alloc)? {
            Pair(_node_ptr, node_ptr2) => {
                let node = Node::new(&alloc, node_ptr2);
                match node_to_bytes(&node) {
                    Ok(serial_data) => Ok(Program::new(serial_data)),
                    Err(error) => Err(Box::new(error)),
                }
            }
            Atom(_buf) => Err("First of non-cons".into()),
        }
    }

    pub fn iter(&self) -> ProgramIter {
        let cloned = self.clone();
        ProgramIter {
            program: cloned,
            current_node: None,
        }
    }
}

impl Into<SerializedProgram> for Program {
    fn into(self) -> SerializedProgram {
        SerializedProgram::from_bytes(&self.serialized)
    }
}

impl From<Vec<u8>> for Program {
    fn from(bytes: Vec<u8>) -> Self {
        SerializedProgram::from_bytes(&bytes)
            .to_program()
            .unwrap_or(Program::new(Vec::new()))
    }
}

impl From<&Vec<u8>> for Program {
    fn from(bytes: &Vec<u8>) -> Self {
        SerializedProgram::from_bytes(bytes)
            .to_program()
            .unwrap_or(Program::new(Vec::new()))
    }
}

impl TryFrom<(Program, Program)> for Program {
    type Error = Box<(dyn StdError + 'static)>;
    fn try_from((first, second): (Program, Program)) -> Result<Self, Self::Error> {
        let mut alloc = Allocator::new();
        let first = node_from_bytes(&mut alloc, &first.serialized.as_slice())?;
        let rest = node_from_bytes(&mut alloc, &second.serialized.as_slice())?;
        match alloc.new_pair(first, rest) {
            Ok(pair) => Ok(SerializedProgram::from_bytes(&node_to_bytes(&Node {
                allocator: &alloc,
                node: pair,
            })?)
            .to_program()
            .unwrap_or(Program::new(Vec::new()))),
            Err(error) => Err(error.1.into()),
        }
    }
}

macro_rules! impl_sized_bytes {
    ($($name: ident);*) => {
        $(
            impl From<$name> for Program {
                fn from(bytes: $name) -> Self {
                    SerializedProgram::from_bytes(&bytes.to_bytes())
                        .to_program()
                        .unwrap_or(Program::new(Vec::new()))
                }
            }
            impl From<&$name> for Program {
                fn from(bytes: &$name) -> Self {
                    SerializedProgram::from_bytes(&bytes.to_bytes())
                        .to_program()
                        .unwrap_or(Program::new(Vec::new()))
                }
            }
        )*
    };
    ()=>{};
}

impl_sized_bytes!(
    UnsizedBytes;
    Bytes4;
    Bytes8;
    Bytes16;
    Bytes32;
    Bytes48;
    Bytes96;
    Bytes192
);

macro_rules! impl_ints {
    ($($name: ident, $size: expr);*) => {
        $(
            impl From<$name> for Program {
                fn from(int_val: $name) -> Self {
                    if int_val == 0 {
                        return Program::new(Vec::new());
                    }
                    let as_ary = int_val.to_be_bytes();
                    let mut as_bytes = as_ary.as_slice();
                    while as_bytes.len() > 1 && as_bytes[0] == ( if as_bytes[1] & 0x80 > 0{0xFF} else {0}) {
                        as_bytes = &as_bytes[1..];
                    }
                    SerializedProgram::from_bytes(&as_bytes.to_vec())
                        .to_program()
                        .unwrap_or(Program::new(Vec::new()))
                }
            }
            impl Into<$name> for Program {
                fn into(self) -> $name {
                    let mut byte_ary: [u8; $size] = [0; $size];
                    byte_ary[..$size].clone_from_slice(&self.serialized);
                    $name::from_be_bytes(byte_ary)
                }
            }
        )*
    };
    ()=>{};
}

impl_ints!(
    u8, 1;
    u16, 2;
    u32, 4;
    u64, 8;
    u128, 16;
    i8, 1;
    i16, 2;
    i32, 4;
    i64, 8;
    i128, 16
);

pub struct ProgramIter {
    program: Program,
    current_node: Option<Program>,
}
impl Iterator for ProgramIter {
    type Item = Program;
    fn next(&mut self) -> Option<Self::Item> {
        let cur_node = match &self.current_node {
            Some(node) => node.clone(),
            None => self.program.clone(),
        };
        match &self.program.null_pointer() {
            Ok(is_nullp) => {
                if !is_nullp {
                    let rtn = match cur_node.first() {
                        Ok(program) => program,
                        Err(_) => {
                            return None;
                        }
                    };
                    self.current_node = match cur_node.rest() {
                        Ok(program) => Some(program),
                        Err(_) => {
                            return None;
                        }
                    };
                    Some(rtn)
                } else {
                    None
                }
            }
            Err(_) => None,
        }
    }
}
impl Program {
    pub fn new(serialized: Vec<u8>) -> Self {
        Program {
            serialized: serialized.clone(),
        }
    }
}
