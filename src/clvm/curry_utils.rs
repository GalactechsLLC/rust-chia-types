// use crate::clvm::serialized_program::SerializedProgram;
// use clvmr::allocator::SExp::Atom;
// use clvmr::allocator::{Allocator, NodePtr};
// use clvmr::node::Node;
// use clvmr::serialize::{node_from_bytes, node_to_bytes};
// use std::collections::HashMap;
// use std::error::Error;
//
// fn assemble_to_program<S: AsRef<str>>(asStr: S) -> SerializedProgram {
//     SerializedProgram::from_bytes(vec![])
// }
//
// const CURRY_OBJ_CODE: SerializedProgram = assemble_to_program("(a (q #a 4 (c 2 (c 5 (c 7 0)))) (c (q (c (q . 2) (c (c (q . 1) 5) (c (a 6 (c 2 (c 11 (q 1)))) 0))) #a (i 5 (q 4 (q . 4) (c (c (q . 1) 9) (c (a 6 (c 2 (c 13 (c 11 0)))) 0))) (q . 11)) 1) 1))");
// const UNCURRY_PATTERN_FUNCTION: SerializedProgram =
//     assemble_to_program("(a (q . (: . function)) (: . core))");
// const UNCURRY_PATTERN_CORE: SerializedProgram =
//     assemble_to_program("(c (q . (: . parm)) (: . core))");
//
// pub async fn uncurry(
//     curried_program: &SerializedProgram,
// ) -> Result<Option<(SerializedProgram, SerializedProgram)>, Box<dyn Error>> {
//     let mut alloc = Allocator::new();
//     let pattern_func =
//         node_from_bytes(&mut alloc, &UNCURRY_PATTERN_FUNCTION.to_bytes().as_slice())?;
//     let pattern_core = node_from_bytes(&mut alloc, &UNCURRY_PATTERN_CORE.to_bytes().as_slice())?;
//     let sexp = node_from_bytes(&mut alloc, &curried_program.to_bytes().as_slice())?;
//     match match_sexp(&mut alloc, pattern_func, sexp, HashMap::new()) {
//         Some(func_results) => {
//             let func = func_results.get("function").unwrap();
//             let mut core = func_results.get("core").unwrap();
//             let mut args = Vec::new();
//             loop {
//                 match match_sexp(
//                     &mut alloc,
//                     pattern_core.clone(),
//                     core.to_owned(),
//                     HashMap::new(),
//                 ) {
//                     Some(core_results) => {
//                         args.push(core_results.get("parm").unwrap());
//                         core = core_results.get("core").unwrap();
//                     }
//                     None => break,
//                 }
//             }
//             match alloc.sexp(*core) {
//                 Atom(buf) => {
//                     let bytes = alloc.buf(buf);
//                     match bytes {
//                         [b"\x81"] => Ok(Some((
//                             SerializedProgram::from_bytes(&node_to_bytes(Node::new(&alloc, func))?),
//                             SerializedProgram::from_bytes(node_to_bytes(Node::new(
//                                 &alloc,
//                                 alloc.new_concat(args.len(), args.as_slice())?,
//                             ))),
//                         ))),
//                         _ => Ok(None),
//                     }
//                 }
//                 _ => Ok(None),
//             }
//         }
//         None => Ok(None),
//     }
// }
//
// // pub async fn curry(
// //     program: &SerializedProgram,
// //     args: &SerializedProgram,
// // ) -> Result<SerializedProgram, Box<dyn Error>> {
// //     let args = program.to((program, args));
// //     r = run_program(CURRY_OBJ_CODE, args);
// //     return r;
// // }
