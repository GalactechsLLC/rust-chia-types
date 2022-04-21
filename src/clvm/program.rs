use crate::blockchain::sized_bytes::*;
use crate::clvm::curry_utils::{curry, uncurry};
use crate::clvm::serialized_program::SerializedProgram;

use clvm_rs::allocator::Allocator as Allocator2;
use clvm_rs::serialize::node_from_bytes as deserialize2;
use clvm_tools_rs::classic::clvm_tools::sha256tree::sha256tree;
use clvmr::allocator::SExp::{Atom, Pair};
use clvmr::allocator::{Allocator, SExp};
use clvmr::node::Node;
use clvmr::serialize::{node_from_bytes, node_to_bytes};
use hex::encode;
use num_bigint::BigInt;
use serde::ser::StdError;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::hash::Hash;
use std::hash::Hasher;

#[derive(Debug)]
pub struct Program {
    pub serialized: Vec<u8>,
    alloc: Allocator,
    nodeptr: i32,
}
impl Program {
    pub fn curry(&self, args: Vec<Program>) -> Result<Program, Box<dyn Error>> {
        let (_cost, program) = curry(&self, args)?;
        Ok(program)
    }

    pub fn uncurry(&self) -> Result<(Program, Program), Box<dyn Error>> {
        let serial_program = SerializedProgram::from_bytes(&self.serialized);
        match uncurry(&serial_program)? {
            Some((program, args)) => Ok((program.to_program()?, args.to_program()?)),
            None => Ok((serial_program.to_program()?, 0.into())),
        }
    }

    pub fn as_atom_list(&mut self) -> Vec<Vec<u8>> {
        let mut rtn: Vec<Vec<u8>> = Vec::new();
        let mut current = self.clone();
        loop {
            match current.as_pair() {
                None => {
                    break;
                }
                Some((first, rest)) => match first.as_vec() {
                    None => {
                        break;
                    }
                    Some(atom) => {
                        rtn.push(atom);
                        current = rest;
                    }
                },
            }
        }
        rtn
    }

    pub fn to_map(self) -> Result<HashMap<Program, Program>, Box<dyn Error>> {
        let mut rtn: HashMap<Program, Program> = HashMap::new();
        let mut cur_node = self;
        loop {
            match cur_node.to_sexp() {
                Atom(_) => break,
                Pair(_, _) => {
                    let pair = cur_node.as_pair().unwrap();
                    cur_node = pair.1;
                    match pair.0.to_sexp() {
                        Atom(_) => {
                            rtn.insert(pair.0.as_atom().unwrap(), Program::new(Vec::new()));
                        }
                        Pair(_, _) => {
                            let inner_pair = pair.0.as_pair().unwrap();
                            rtn.insert(inner_pair.0, inner_pair.1);
                        }
                    }
                }
            }
        }
        Ok(rtn)
    }

    pub fn to_sexp(&self) -> SExp {
        self.alloc.sexp(self.nodeptr)
    }

    pub fn to_node(&self) -> Node {
        Node::new(&self.alloc, self.nodeptr).clone()
    }

    pub fn is_atom(&self) -> bool {
        self.as_atom().is_some()
    }

    pub fn is_pair(&self) -> bool {
        self.as_pair().is_some()
    }

    pub fn as_atom(&self) -> Option<Program> {
        match self.to_sexp() {
            Atom(_) => Some(Program::new(self.alloc.atom(self.nodeptr).to_vec())),
            _ => None,
        }
    }
    pub fn as_vec(&self) -> Option<Vec<u8>> {
        match self.to_sexp() {
            Atom(_) => Some(self.alloc.atom(self.nodeptr).to_vec()),
            _ => None,
        }
    }

    pub fn as_pair(&self) -> Option<(Program, Program)> {
        match self.to_sexp() {
            Pair(p1, p2) => {
                let left_node = Node::new(&self.alloc, p1);
                let right_node = Node::new(&self.alloc, p2);
                let left = match node_to_bytes(&left_node) {
                    Ok(serial_data) => Program::new(serial_data),
                    Err(_) => Program::new(Vec::new()),
                };
                let right = match node_to_bytes(&right_node) {
                    Ok(serial_data) => Program::new(serial_data),
                    Err(_) => Program::new(Vec::new()),
                };
                Some((left, right))
            }
            _ => None,
        }
    }

    pub fn cons(&self, other: &Program) -> Program {
        let mut alloc = Allocator::new();
        let first = match node_from_bytes(&mut alloc, &self.serialized.as_slice()) {
            Ok(ptr) => ptr,
            Err(_) => alloc.null(),
        };
        let rest = match node_from_bytes(&mut alloc, &other.serialized.as_slice()) {
            Ok(ptr) => ptr,
            Err(_) => alloc.null(),
        };
        match alloc.new_pair(first, rest) {
            Ok(pair) => {
                let node = Node::new(&alloc, pair);
                let node_bytes = node_to_bytes(&node).unwrap();
                Program::new(node_bytes)
            }
            Err(_) => Program::null(),
        }
    }

    pub fn as_int(&self) -> Result<BigInt, Box<dyn Error>> {
        match &self.as_atom() {
            Some(atom) => Ok(BigInt::from_signed_bytes_be(
                atom.as_vec().unwrap().as_slice(),
            )),
            None => {
                log::debug!("BAD INT: {:?}", self.serialized);
                Err("Program is Pair not Atom".into())
            }
        }
    }

    pub fn first(&self) -> Result<Program, Box<dyn Error>> {
        match self.as_pair() {
            Some((p1, _)) => Ok(p1),
            _ => Err("first of non-cons".into()),
        }
    }

    pub fn rest(&self) -> Result<Program, Box<dyn Error>> {
        match self.as_pair() {
            Some((_, p2)) => Ok(p2),
            _ => Err("rest of non-cons".into()),
        }
    }

    pub fn iter(&self) -> ProgramIter {
        ProgramIter {
            node: Node::new(&self.alloc, self.nodeptr).clone().into_iter(),
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
        let mut alloc = Allocator::new();
        let atom = match alloc.new_atom(bytes.as_slice()) {
            Ok(ptr) => ptr,
            Err(_) => alloc.null(),
        };
        let node = Node::new(&alloc, atom);
        let node_bytes = match node_to_bytes(&node) {
            Ok(n_bytes) => n_bytes,
            Err(_) => Vec::new(),
        };
        let prog = Program {
            serialized: node_bytes,
            alloc: alloc,
            nodeptr: atom,
        };
        prog
    }
}

impl From<&Vec<u8>> for Program {
    fn from(bytes: &Vec<u8>) -> Self {
        let mut alloc = Allocator::new();
        let atom = match alloc.new_atom(bytes.as_slice()) {
            Ok(ptr) => ptr,
            Err(_) => alloc.null(),
        };
        let node = Node::new(&alloc, atom);
        let node_bytes = match node_to_bytes(&node) {
            Ok(n_bytes) => n_bytes,
            Err(_) => Vec::new(),
        };
        let prog = Program {
            serialized: node_bytes,
            alloc: alloc,
            nodeptr: atom,
        };
        prog
    }
}

impl TryFrom<(Program, Program)> for Program {
    type Error = Box<(dyn StdError + 'static)>;
    fn try_from((first, second): (Program, Program)) -> Result<Self, Self::Error> {
        let mut alloc = Allocator::new();
        let first = node_from_bytes(&mut alloc, &first.serialized.as_slice())?;
        let rest = node_from_bytes(&mut alloc, &second.serialized.as_slice())?;
        match alloc.new_pair(first, rest) {
            Ok(pair) => {
                let node = Node::new(&alloc, pair);
                let node_bytes = node_to_bytes(&node)?;
                Ok(Program::new(node_bytes))
            }
            Err(error) => Err(error.1.into()),
        }
    }
}

macro_rules! impl_sized_bytes {
    ($($name: ident, $size:expr);*) => {
        $(
            impl From<$name> for Program {
                fn from(bytes: $name) -> Self {
                    bytes.to_bytes().into()
                }
            }
            impl From<&$name> for Program {
                fn from(bytes: &$name) -> Self {
                    bytes.to_bytes().into()
                }
            }
            impl Into<$name> for Program {
                fn into(self) -> $name {
                    let vec_len = self.serialized.len();
                    if vec_len == $size + 1 {
                        $name::new(self.serialized[1..].to_vec())
                    } else {
                        $name::new(self.serialized)
                    }
                }
            }
            impl Into<$name> for &Program {
                fn into(self) -> $name {
                    let vec_len = self.serialized.len();
                    if vec_len == $size + 1 {
                        $name::new(self.serialized[1..].to_vec())
                    } else {
                        $name::new(self.serialized.clone())
                    }
                }
            }
        )*
    };
    ()=>{};
}

impl_sized_bytes!(
    UnsizedBytes, 0;
    Bytes4, 4;
    Bytes8, 8;
    Bytes16, 16;
    Bytes32, 32;
    Bytes48, 48;
    Bytes96, 96;
    Bytes192, 192
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
                    as_bytes.to_vec().into()
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

pub struct ProgramIter<'a> {
    node: Node<'a>,
}
impl Iterator for ProgramIter<'_> {
    type Item = Program;
    fn next(&mut self) -> Option<Self::Item> {
        match self.node.next() {
            Some(m_node) => match node_to_bytes(&m_node) {
                Ok(bytes) => {
                    let prog = Program::new(bytes);
                    Some(prog)
                }
                Err(_) => None,
            },
            None => None,
        }
    }
}

impl Clone for Program {
    fn clone(&self) -> Self {
        Program::new(self.serialized.clone())
    }
}

impl Hash for Program {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.serialized.hash(state);
    }
}

impl PartialEq for Program {
    fn eq(&self, other: &Self) -> bool {
        self.serialized == other.serialized
    }
}
impl Eq for Program {}

impl fmt::Display for Program {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({})", encode(&self.serialized))
    }
}

impl Program {
    pub fn new(serialized: Vec<u8>) -> Self {
        let alloc = Allocator::new();
        let null = alloc.null();
        let mut prog = Program {
            serialized: serialized.clone(),
            alloc: alloc,
            nodeptr: null,
        };
        prog.set_node();
        prog
    }
    pub fn null() -> Self {
        let alloc = Allocator::new();
        let null = alloc.null();
        let serial = match node_to_bytes(&Node::new(&alloc, null)) {
            Ok(bytes) => bytes,
            Err(_) => vec![],
        };
        let mut prog = Program {
            serialized: serial,
            alloc: alloc,
            nodeptr: null,
        };
        prog.set_node();
        prog
    }
    fn set_node(&mut self) {
        self.nodeptr = match node_from_bytes(&mut self.alloc, &self.serialized) {
            Ok(node) => node,
            Err(_) => self.alloc.null(),
        };
    }
    pub fn tree_hash(&self) -> Bytes32 {
        let mut alloc2 = Allocator2::new();
        let nodeptr = match deserialize2(&mut alloc2, &self.serialized) {
            Ok(node) => node,
            Err(_) => alloc2.null(),
        };
        Bytes32::new(sha256tree(&mut alloc2, nodeptr).raw())
    }
}
