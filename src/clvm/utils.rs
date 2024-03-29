use crate::blockchain::sized_bytes::Bytes32;
use chia::gen::flags::{COND_ARGS_NIL, COND_CANON_INTS, NO_UNKNOWN_CONDS};
use clvmr::allocator::SExp::Atom;
use clvmr::allocator::SExp::Pair;
use clvmr::allocator::{Allocator, NodePtr};
use clvmr::chia_dialect::{NO_NEG_DIV, NO_UNKNOWN_OPS};
use sha2::{Digest, Sha256, Sha512};
use std::collections::HashSet;
use std::error::Error;

pub const MEMPOOL_MODE: u32 =
    NO_NEG_DIV | COND_CANON_INTS | NO_UNKNOWN_CONDS | NO_UNKNOWN_OPS | COND_ARGS_NIL;

pub const INFINITE_COST: u64 = 0x7FFFFFFFFFFFFFFF;

pub fn tree_hash(
    alloc: &Allocator,
    node_ptr: NodePtr,
    precalculated: &HashSet<Bytes32>,
) -> Result<Bytes32, Box<dyn Error>> {
    match alloc.sexp(node_ptr) {
        Atom(_buf) => {
            let atom = alloc.atom(node_ptr);
            if precalculated.contains(&Vec::from(atom).into()) {
                Ok(Vec::from(atom).into())
            } else {
                let mut byte_buf = Vec::new();
                byte_buf.extend([b'1']);
                byte_buf.extend(atom);
                Ok(hash_256(byte_buf).into())
            }
        }
        Pair(first, rest) => {
            let mut byte_buf = Vec::new();
            byte_buf.extend([b'2']);
            byte_buf.append(&mut tree_hash(&alloc, first, &precalculated)?.into());
            byte_buf.append(&mut tree_hash(&alloc, rest, &precalculated)?.into());
            Ok(hash_256(byte_buf).into())
        }
    }
}

pub fn hash_256(input: Vec<u8>) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(input);
    hasher.finalize().to_vec()
}

pub fn hash_512(input: Vec<u8>) -> Vec<u8> {
    let mut hasher = Sha512::new();
    hasher.update(input);
    hasher.finalize().to_vec()
}
