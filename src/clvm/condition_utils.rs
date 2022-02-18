use crate::blockchain::announcement::Announcement;
use crate::blockchain::coin::Coin;
use crate::blockchain::condition_opcode::ConditionOpcode;
use crate::blockchain::condition_with_args::ConditionWithArgs;
use crate::blockchain::sized_bytes::Bytes32;
use crate::blockchain::sized_bytes::SizedBytes;
use crate::blockchain::utils::atom_to_uint;
use crate::clvm::program::Program;
use crate::clvm::serialized_program::SerializedProgram;

use clvmr::allocator::Allocator;
use clvmr::node::Node;
use clvmr::serialize::node_to_bytes;
use std::collections::HashMap;
use std::collections::HashSet;
use std::error::Error;

pub fn parse_sexp_to_condition(sexp: &mut Program) -> Result<ConditionWithArgs, Box<dyn Error>> {
    match sexp.as_atom_list() {
        Some(as_atoms) => {
            if as_atoms.len() < 1 {
                Err("Invalid Condition".into())
            } else {
                match as_atoms.split_first() {
                    Some((first, rest)) => match ConditionOpcode::from_u8(first[0]) {
                        Ok(opcode) => Ok(ConditionWithArgs {
                            opcode,
                            vars: Vec::from(rest),
                        }),
                        Err(_error) => Err("Invalid Condition".into()),
                    },
                    None => Err("Invalid Condition".into()),
                }
            }
        }
        None => Err("Invalid Condition".into()),
    }
}

pub fn parse_sexp_to_conditions(
    sexp: &SerializedProgram,
) -> Result<Vec<ConditionWithArgs>, Box<dyn Error>> {
    let mut results = Vec::new();
    for mut arg in sexp.to_program() {
        match parse_sexp_to_condition(&mut arg) {
            Ok(condition) => {
                results.push(condition);
            }
            Err(error) => return Err(error),
        }
    }
    Ok(results)
}

pub fn conditions_by_opcode(
    conditions: Vec<ConditionWithArgs>,
) -> HashMap<ConditionOpcode, Vec<ConditionWithArgs>> {
    let mut hm: HashMap<ConditionOpcode, Vec<ConditionWithArgs>> = HashMap::new();
    for cvp in conditions {
        match hm.get_mut(&cvp.opcode) {
            Some(list) => {
                list.push(cvp.clone());
            }
            None => {
                hm.insert(cvp.opcode.clone(), vec![cvp.clone()]);
            }
        }
    }
    return hm;
}

pub fn created_outputs_for_conditions_dict(
    conditions_dict: HashMap<ConditionOpcode, Vec<ConditionWithArgs>>,
    input_coin_name: Bytes32,
) -> Vec<Coin> {
    let mut output_coins = Vec::new();
    match conditions_dict.get(&ConditionOpcode::CreateCoin) {
        Some(args) => {
            for cvp in args {
                let puz_hash = cvp.vars[0].clone();
                let amount = atom_to_uint(&cvp.vars[1]);
                let coin = Coin {
                    parent_coin_info: input_coin_name.clone(),
                    puzzle_hash: Bytes32::from_bytes(puz_hash),
                    amount,
                };
                output_coins.push(coin);
            }
        }
        None => {}
    }
    output_coins
}

pub fn coin_announcements_for_conditions_dict(
    conditions_dict: HashMap<ConditionOpcode, Vec<ConditionWithArgs>>,
    input_coin: Coin,
) -> HashSet<Announcement> {
    let mut output_announcements = HashSet::new();
    match conditions_dict.get(&ConditionOpcode::CreateCoinAnnouncement) {
        Some(args) => {
            for cvp in args {
                let message = cvp.vars[0].clone(); //TODO come back and check panic
                                                   // assert len(message) < = 1024
                let announcement = Announcement {
                    origin_info: input_coin.name(),
                    message: message,
                };
                output_announcements.insert(announcement);
            }
        }
        None => {}
    }
    output_announcements
}

pub fn puzzle_announcements_for_conditions_dict(
    conditions_dict: HashMap<ConditionOpcode, Vec<ConditionWithArgs>>,
    input_coin: Coin,
) -> HashSet<Announcement> {
    let mut output_announcements = HashSet::new();
    match conditions_dict.get(&ConditionOpcode::CreatePuzzleAnnouncement) {
        Some(args) => {
            for cvp in args {
                let message = cvp.vars[0].clone(); //TODO come back and check panic
                                                   // assert len(message) < = 1024
                let announcement = Announcement {
                    origin_info: input_coin.puzzle_hash.clone(),
                    message: message,
                };
                output_announcements.insert(announcement);
            }
        }
        None => {}
    }
    output_announcements
}

pub fn coin_announcement_names_for_conditions_dict(
    conditions_dict: HashMap<ConditionOpcode, Vec<ConditionWithArgs>>,
    input_coin: Coin,
) -> Vec<Bytes32> {
    let mut output = Vec::new();
    for announcement in coin_announcements_for_conditions_dict(conditions_dict, input_coin) {
        output.push(announcement.name());
    }
    output
}

pub fn puzzle_announcement_names_for_conditions_dict(
    conditions_dict: HashMap<ConditionOpcode, Vec<ConditionWithArgs>>,
    input_coin: Coin,
) -> Vec<Bytes32> {
    let mut output = Vec::new();
    for announcement in puzzle_announcements_for_conditions_dict(conditions_dict, input_coin) {
        output.push(announcement.name());
    }
    output
}

pub fn conditions_dict_for_solution(
    puzzle_reveal: &SerializedProgram,
    solution: &SerializedProgram,
    max_cost: u64,
) -> Result<(HashMap<ConditionOpcode, Vec<ConditionWithArgs>>, u64), Box<dyn Error>> {
    match conditions_for_solution(puzzle_reveal, solution, max_cost) {
        Ok((result, cost)) => Ok((conditions_by_opcode(result), cost)),
        Err(error) => Err(error),
    }
}

pub fn conditions_for_solution(
    puzzle_reveal: &SerializedProgram,
    solution: &SerializedProgram,
    max_cost: u64,
) -> Result<(Vec<ConditionWithArgs>, u64), Box<dyn Error>> {
    let mut allocator = Allocator::new();
    match puzzle_reveal.run_with_cost(&mut allocator, max_cost, solution.to_bytes()) {
        Ok((cost, r)) => {
            let node = Node::new(&allocator, r);
            match node_to_bytes(&node) {
                Ok(byte_data) => {
                    let serial_program = SerializedProgram::from_bytes(&byte_data);
                    match parse_sexp_to_conditions(&serial_program) {
                        Ok(conditions) => Ok((conditions, cost)),
                        Err(error) => Err(error),
                    }
                }
                Err(error) => Err(Box::new(error)),
            }
        }
        Err(error) => Err(error),
    }
}

//
// pub fn pkm_pairs(npc_list: Vec<[NPC>, additional_data: bytes) -> (Vec<Bytes48>, Vec<Vec<u8>>){
//     let ret:(Vec<Bytes48>, Vec<Vec<u8>>) = (Vec::new(), Vec::new());
//     for npc in npc_list:
//     for opcode, l in npc.conditions:
//     if opcode == ConditionOpcode.AGG_SIG_UNSAFE:
//     for cwa in l:
//     assert len(cwa.vars) == 2
//     assert len(cwa.vars[0]) == 48 and len(cwa.vars[1]) <= 1024
//     assert cwa.vars[0] is not None and cwa.vars[1] is not None
//     ret[0].append(bytes48(cwa.vars[0]))
//     ret[1].append(cwa.vars[1])
//     elif opcode == ConditionOpcode.AGG_SIG_ME:
//     for cwa in l:
//     assert len(cwa.vars) == 2
//     assert len(cwa.vars[0]) == 48 and len(cwa.vars[1]) < = 1024
//     assert cwa.vars[0] is not None and cwa.vars[1] is not None
//     ret[0].append(bytes48(cwa.vars[0]))
//     ret[1].append(cwa.vars[1] + npc.coin_name + additional_data)
//     return ret
// }
//
//
// pub fn pkm_pairs_for_conditions_dict(
//     conditions_dict: HashMap<ConditionOpcode, Vec<ConditionWithArgs>>, coin_name: bytes32, additional_data: bytes
// ) -> Vec<(Bytes48, Vec<u8>)> {
//     assert coin_name is not None
//     ret: List[Tuple[bytes48, bytes]] = []
//
//     for cwa in conditions_dict.get(ConditionOpcode.AGG_SIG_UNSAFE, []):
//     assert len(cwa.vars) == 2
//     assert len(cwa.vars[0]) == 48 and len(cwa.vars[1]) <= 1024
//     assert cwa.vars[0] is not None and cwa.vars[1] is not None
//     ret.append((bytes48(cwa.vars[0]), cwa.vars[1]))
//
//     for cwa in conditions_dict.get(ConditionOpcode.AGG_SIG_ME, []):
//     assert len(cwa.vars) == 2
//     assert len(cwa.vars[0]) == 48 and len(cwa.vars[1]) <= 1024
//     assert cwa.vars[0] is not None and cwa.vars[1] is not None
//     ret.append((bytes48(cwa.vars[0]), cwa.vars[1] + coin_name + additional_data))
//     return ret
// }
//
