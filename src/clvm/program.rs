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
use std::hash::Hash;
use std::hash::Hasher;

pub struct Program {
    pub serialized: Vec<u8>,
    alloc: Allocator,
    nodeptr: i32,
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

    pub fn as_atom_list(&mut self) -> Option<Vec<Vec<u8>>> {
        let mut rtn: Vec<Vec<u8>> = Vec::new();
        match node_from_bytes(&mut self.alloc, &self.serialized) {
            Ok(program) => {
                let mut cur_node = program;
                loop {
                    let sexp = self.alloc.sexp(cur_node);
                    match sexp {
                        Atom(_buf) => break,
                        Pair(node_ptr, node_ptr2) => {
                            rtn.extend([Vec::from(self.alloc.atom(node_ptr))]);
                            cur_node = node_ptr2;
                        }
                    }
                }
                Some(rtn)
            }
            Err(_error) => None,
        }
    }

    // fn node_to_program(&mut alloc: Allocator, node_ptr: NodePtr) -> Program {
    //     match alloc.sexp(node_ptr) {
    //         Atom(buf) => Program::new(alloc.buf(&buf).to_vec()),
    //         Pair(key, value) => (
    //             node_to_program(&mut alloc, key),
    //             node_to_program(&mut alloc, value),
    //         )
    //             .into(),
    //     }
    // }

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

    pub fn get_tree_hash(&self, args: &Vec<Bytes32>) -> Result<Bytes32, Box<dyn Error>> {
        SerializedProgram::from_bytes(&self.serialized).get_tree_hash(args)
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

    pub fn as_atom(&self) -> Option<Program> {
        match self.to_sexp() {
            Atom(_) => Some(Program::new(self.alloc.atom(self.nodeptr).to_vec())),
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

    pub fn as_int(&self) -> Result<BigInt, Box<dyn Error>> {
        match &self.as_atom() {
            Some(atom) => Ok(BigInt::from_signed_bytes_be(atom.serialized.as_slice())),
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

    // pub fn first(&self) -> Result<Program, Box<dyn Error>> {
    //     let alloc = Allocator::new();
    //     match self.to_sexp() {
    //         Pair(node_ptr, _node_ptr2) => {
    //             let node = Node::new(&alloc, node_ptr);
    //             match node_to_bytes(&node) {
    //                 Ok(serial_data) => Ok(Program::new(serial_data)),
    //                 Err(error) => Err(Box::new(error)),
    //             }
    //         }
    //         Atom(_buf) => Err("First of non-cons".into()),
    //     }
    // }
    //
    // pub fn rest(&mut self) -> Result<Program, Box<dyn Error>> {
    //     match self.to_sexp() {
    //         Pair(_node_ptr, node_ptr2) => {
    //             match node_to_bytes(
    //                 &Node::new(&mut self.alloc, node_ptr2)
    //                     .rest()
    //                     .map_err(|e| e.1)?,
    //             ) {
    //                 Ok(serial_data) => Ok(Program::new(serial_data)),
    //                 Err(error) => Err(Box::new(error)),
    //             }
    //         }
    //         Atom(_buf) => Err("First of non-cons".into()),
    //     }
    // }

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

pub struct ProgramIter<'a> {
    node: Node<'a>,
}
impl Iterator for ProgramIter<'_> {
    type Item = Program;
    fn next(&mut self) -> Option<Self::Item> {
        match self.node.next() {
            Some(m_node) => match node_to_bytes(&m_node) {
                Ok(bytes) => Some(Program::from(bytes)),
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
    fn set_node(&mut self) {
        self.nodeptr = match node_from_bytes(&mut self.alloc, &self.serialized) {
            Ok(node) => node,
            Err(_) => self.alloc.null(),
        };
    }
}
