use clvmr::allocator::SExp::{Atom, Pair};
use clvmr::allocator::{Allocator, NodePtr, SExp};
use clvmr::node::Node;
use clvmr::serialize::{node_from_bytes, node_to_bytes};
use std::collections::HashMap;
use std::error::Error;
use std::str;

#[derive(Clone)]
pub struct Program {
    serialized: Vec<u8>,
}
impl Program {
    // pub fn curry() -> {
    //
    // }
    //
    // pub fn uncurry() -> {
    //
    // }

    fn null_pointer(&self, node_ptr: Option<NodePtr>) -> bool {
        let alloc = Allocator::new();
        match node_ptr {
            Some(node_ptr) => {
                if node_ptr < 0 {
                    let atom = alloc.atom(node_ptr);
                    atom.len() == 0
                } else {
                    false
                }
            }
            None => false,
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

    pub fn to_sexp(&self, alloc: &mut Allocator) -> Result<SExp, Box<dyn Error>> {
        let node = node_from_bytes(alloc, &self.serialized)?;
        Ok(alloc.sexp(node))
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
        let mut alloc = Allocator::new();
        let cloned = self.clone();
        ProgramIter {
            program: cloned,
            current_node: match node_from_bytes(&mut alloc, &self.serialized) {
                Ok(node_ptr) => Some(node_ptr),
                Err(_error) => None,
            },
            allocator: alloc,
        }
    }
}
pub struct ProgramIter {
    program: Program,
    current_node: Option<NodePtr>,
    allocator: Allocator,
}
impl Iterator for ProgramIter {
    type Item = SExp;
    fn next(&mut self) -> Option<Self::Item> {
        let cur_node = match self.current_node {
            Some(node) => Some(node),
            None => match node_from_bytes(&mut self.allocator, &self.program.serialized) {
                Ok(node_ptr) => Some(node_ptr),
                Err(_error) => None,
            },
        };
        if !self.program.null_pointer(cur_node) {
            let sexp = self.allocator.sexp(self.current_node.unwrap());
            match sexp {
                Atom(_buf) => None,
                Pair(node_ptr, node_ptr2) => {
                    self.current_node = Some(node_ptr2);
                    Some(self.allocator.sexp(node_ptr))
                }
            }
        } else {
            None
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
