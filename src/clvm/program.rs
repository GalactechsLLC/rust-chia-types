use clvmr::allocator::SExp::{Atom, Pair};
use clvmr::allocator::{Allocator, NodePtr, SExp};
use clvmr::serialize::node_from_bytes;
use std::error::Error;

pub struct Program {
    allocator: Allocator,
    serialized: Vec<u8>,
    current_node: Option<NodePtr>,
}
impl Program {
    fn null_pointer(&self) -> bool {
        match self.current_node {
            Some(node_ptr) => {
                if node_ptr < 0 {
                    let atom = self.allocator.atom(node_ptr);
                    atom.len() == 0
                } else {
                    false
                }
            }
            None => false,
        }
    }

    pub(crate) fn as_atom_list(&mut self) -> Option<Vec<Vec<u8>>> {
        let mut rtn: Vec<Vec<u8>> = Vec::new();
        match node_from_bytes(&mut self.allocator, &mut self.serialized) {
            Ok(program) => {
                let mut cur_node = program;
                loop {
                    let sexp = self.allocator.sexp(cur_node);
                    match sexp {
                        Atom(_buf) => break,
                        Pair(node_ptr, node_ptr2) => {
                            rtn.extend([Vec::from(self.allocator.atom(node_ptr))]);
                            cur_node = node_ptr2;
                        }
                    }
                }
                Some(rtn)
            }
            Err(_error) => None,
        }
    }
}
impl Iterator for Program {
    type Item = SExp;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.null_pointer() {
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
    pub fn new(serial_data: Vec<u8>) -> Result<Self, Box<dyn Error>> {
        let mut program = Program {
            allocator: Allocator::new(),
            serialized: serial_data.clone(),
            current_node: None,
        };
        match program.init() {
            Ok(()) => Ok(program),
            Err(error) => Err(error),
        }
    }

    fn init(&mut self) -> Result<(), Box<dyn Error>> {
        self.current_node = Some(node_from_bytes(&mut self.allocator, &self.serialized)?);
        Ok(())
    }
}
