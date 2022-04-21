use crate::clvm::program::Program;
use crate::clvm::serialized_program::SerializedProgram;
use clvm_rs::allocator::Allocator as Allocator2;
use clvm_rs::node::Node as Node2;
use clvm_rs::serialize::node_to_bytes as serialize2;
use clvm_tools_rs::classic::clvm::__type_compatibility__::{Bytes, BytesFromType};
use clvm_tools_rs::classic::clvm_tools::binutils::assemble as chia_assemble;
use clvmr::allocator::Allocator;
use clvmr::allocator::NodePtr;
use clvmr::allocator::SExp;
use clvmr::allocator::SExp::Atom;
use clvmr::cost::Cost;
use clvmr::node::Node;
use clvmr::serialize::{node_from_bytes, node_to_bytes};
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::error::Error;

pub fn assemble(input_text: &str) -> SerializedProgram {
    let mut allocator = Allocator2::new();
    let ptr = chia_assemble(&mut allocator, &input_text.to_string()).unwrap();
    let result = serialize2(&Node2::new(&mut allocator, ptr))
        .map_err(|e| e.to_string().as_bytes().to_vec())
        .unwrap();
    SerializedProgram::from_bytes(&result)
}

lazy_static! {
    pub static ref CURRY_OBJ_CODE: SerializedProgram = assemble("(a (q #a 4 (c 2 (c 5 (c 7 0)))) (c (q (c (q . 2) (c (c (q . 1) 5) (c (a 6 (c 2 (c 11 (q 1)))) 0))) #a (i 5 (q 4 (q . 4) (c (c (q . 1) 9) (c (a 6 (c 2 (c 13 (c 11 0)))) 0))) (q . 11)) 1) 1))");
    pub static ref UNCURRY_PATTERN_FUNCTION: SerializedProgram = assemble("(a (q . (: . function)) (: . core))");
    pub static ref UNCURRY_PATTERN_CORE: SerializedProgram = assemble("(c (q . (: . parm)) (: . core))");
}

const BYTE_MATCH: [u8; 1] = [81 as u8];
const ATOM_MATCH: [u8; 1] = ['$' as u8];
const SEXP_MATCH: [u8; 1] = [':' as u8];

pub fn uncurry(
    curried_program: &SerializedProgram,
) -> Result<Option<(SerializedProgram, SerializedProgram)>, Box<dyn Error>> {
    let mut alloc = Allocator::new();
    let pattern_func =
        node_from_bytes(&mut alloc, &UNCURRY_PATTERN_FUNCTION.to_bytes().as_slice())?;
    let pattern_core = node_from_bytes(&mut alloc, &UNCURRY_PATTERN_CORE.to_bytes().as_slice())?;
    let sexp = node_from_bytes(&mut alloc, &curried_program.to_bytes().as_slice())?;
    match match_sexp(&mut alloc, pattern_func, sexp, HashMap::new()) {
        Some(func_results) => {
            let func = *func_results.get("function").unwrap();
            let mut core = *func_results.get("core").unwrap();
            let mut args: Vec<NodePtr> = Vec::new();
            loop {
                match match_sexp(&mut alloc, pattern_core.clone(), core, HashMap::new()) {
                    Some(core_results) => {
                        args.push(*core_results.get("parm").unwrap());
                        core = core_results.get("core").unwrap().clone();
                    }
                    None => break,
                }
            }
            match &alloc.sexp(core) {
                Atom(buf) => {
                    let bytes = alloc.buf(buf).to_vec();
                    let _byte_match_vec = Vec::from(BYTE_MATCH);
                    match bytes {
                        _byte_match_vec => {
                            let node_ptr = match &alloc.new_concat(args.len(), args.as_slice()) {
                                Ok(value) => *value,
                                Err(error) => {
                                    return Err(error.1.clone().into());
                                }
                            };
                            Ok(Some((
                                SerializedProgram::from_bytes(&node_to_bytes(&Node::new(
                                    &alloc, func,
                                ))?),
                                SerializedProgram::from_bytes(&node_to_bytes(&Node::new(
                                    &mut alloc, node_ptr,
                                ))?),
                            )))
                        }
                    }
                }
                _ => Ok(None),
            }
        }
        None => Ok(None),
    }
}

pub fn curry<'a>(program: &Program, args: Vec<Program>) -> Result<(Cost, Program), Box<dyn Error>> {
    let mut alloc = Allocator::new();
    let args = make_args(args);
    let pair: Program = program.cons(&args);
    let cur_prog = CURRY_OBJ_CODE.clone();
    let (cost, result) = cur_prog.run_with_cost(&mut alloc, Cost::MAX, &pair)?;
    let prog = Node::new(&alloc, result);
    let bytes = node_to_bytes(&prog)?;
    Ok((cost, Program::new(bytes)))
}

fn make_args(args: Vec<Program>) -> Program {
    if args.len() == 0 {
        return Program::null();
    }
    let mut rtn = args.last().unwrap().cons(&Program::null());
    let mut rest = args.clone();
    rest.reverse();
    for arg in &rest[1..=rest.len() - 1] {
        rtn = arg.cons(&rtn);
    }
    rtn
}

pub fn match_sexp<'a>(
    allocator: &'a mut Allocator,
    pattern: NodePtr,
    sexp: NodePtr,
    known_bindings: HashMap<String, NodePtr>,
) -> Option<HashMap<String, NodePtr>> {
    /*
     * Determine if sexp matches the pattern, with the given known bindings already applied.
     * Returns None if no match, or a (possibly empty) dictionary of bindings if there is a match
     * Patterns look like this:
     * ($ . $) matches the literal "$", no bindings (mostly useless)
     * (: . :) matches the literal ":", no bindings (mostly useless)
     * ($ . A) matches B if B is an atom; and A is bound to B
     * (: . A) matches B always; and A is bound to B
     * (A . B) matches (C . D) if A matches C and B matches D
     *         and bindings are the unification (as long as unification is possible)
     */

    match (allocator.sexp(pattern), allocator.sexp(sexp)) {
        (SExp::Atom(pat_buf), SExp::Atom(sexp_buf)) => {
            let sexp_bytes = allocator.buf(&sexp_buf).to_vec();
            if allocator.buf(&pat_buf).to_vec() == sexp_bytes {
                return Some(known_bindings);
            } else {
                return None;
            }
        }
        (SExp::Pair(pleft, pright), _) => match (allocator.sexp(pleft), allocator.sexp(pright)) {
            (SExp::Atom(pat_left), SExp::Atom(pat_right)) => {
                let pat_right_bytes = allocator.buf(&pat_right).to_vec();
                let pat_left_bytes = allocator.buf(&pat_left).to_vec();

                match allocator.sexp(sexp) {
                    SExp::Atom(sexp_buf) => {
                        let sexp_bytes = allocator.buf(&sexp_buf).to_vec();
                        if pat_left_bytes == ATOM_MATCH.to_vec() {
                            if pat_right_bytes == ATOM_MATCH.to_vec() {
                                if sexp_bytes == ATOM_MATCH.to_vec() {
                                    return Some(HashMap::new());
                                }
                                return None;
                            }

                            return unify_bindings(
                                allocator,
                                known_bindings,
                                &pat_right_bytes,
                                sexp,
                            );
                        }
                        if pat_left_bytes == SEXP_MATCH.to_vec() {
                            if pat_right_bytes == SEXP_MATCH.to_vec() {
                                if sexp_bytes == SEXP_MATCH.to_vec() {
                                    return Some(HashMap::new());
                                }
                            }

                            return unify_bindings(
                                allocator,
                                known_bindings,
                                &pat_right_bytes,
                                sexp,
                            );
                        }

                        return None;
                    }
                    SExp::Pair(sleft, sright) => {
                        if pat_left_bytes == SEXP_MATCH.to_vec() {
                            if pat_right_bytes != SEXP_MATCH.to_vec() {
                                return unify_bindings(
                                    allocator,
                                    known_bindings,
                                    &pat_right_bytes,
                                    sexp,
                                );
                            }
                        }

                        return match_sexp(allocator, pleft, sleft, known_bindings).and_then(
                            |new_bindings| {
                                return match_sexp(allocator, pright, sright, new_bindings);
                            },
                        );
                    }
                }
            }
            _ => match allocator.sexp(sexp) {
                SExp::Atom(_) => {
                    return None;
                }
                SExp::Pair(sleft, sright) => {
                    return match_sexp(allocator, pleft, sleft, known_bindings).and_then(
                        |new_bindings| {
                            return match_sexp(allocator, pright, sright, new_bindings);
                        },
                    );
                }
            },
        },
        (SExp::Atom(_), _) => {
            return None;
        }
    }
}

pub fn unify_bindings<'a>(
    allocator: &'a mut Allocator,
    bindings: HashMap<String, NodePtr>,
    new_key: &Vec<u8>,
    new_value: NodePtr,
) -> Option<HashMap<String, NodePtr>> {
    /*
     * Try to add a new binding to the list, rejecting it if it conflicts
     * with an existing binding.
     */
    let new_key_str = Bytes::new(Some(BytesFromType::Raw(new_key.to_vec()))).decode();
    match bindings.get(&new_key_str) {
        Some(binding) => {
            if !equal_to(allocator, *binding, new_value) {
                return None;
            }
            return Some(bindings);
        }
        _ => {
            let mut new_bindings = bindings.clone();
            new_bindings.insert(new_key_str, new_value);
            return Some(new_bindings);
        }
    }
}

pub fn equal_to<'a>(allocator: &'a mut Allocator, first_: NodePtr, second_: NodePtr) -> bool {
    let mut first = first_;
    let mut second = second_;

    loop {
        match (allocator.sexp(first), allocator.sexp(second)) {
            (SExp::Atom(fbuf), SExp::Atom(sbuf)) => {
                let fvec = allocator.buf(&fbuf).to_vec();
                let svec = allocator.buf(&sbuf).to_vec();
                return fvec == svec;
            }
            (SExp::Pair(ff, fr), SExp::Pair(rf, rr)) => {
                if !equal_to(allocator, ff, rf) {
                    return false;
                }
                first = fr;
                second = rr;
            }
            _ => {
                return false;
            }
        }
    }
}
