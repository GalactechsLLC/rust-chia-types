use crate::blockchain::coin::Coin;
use crate::blockchain::condition_opcode::ConditionOpcode;
use crate::blockchain::sized_bytes::Bytes32;
use crate::clvm::condition_utils::conditions_dict_for_solution;
use crate::clvm::condition_utils::created_outputs_for_conditions_dict;
use crate::clvm::serialized_program::SerializedProgram;

pub fn additions_for_solution(
    coin_name: Bytes32,
    puzzle_reveal: &SerializedProgram,
    solution: &SerializedProgram,
    max_cost: u64,
) -> Vec<Coin> {
    match conditions_dict_for_solution(puzzle_reveal, solution, max_cost) {
        Ok((map, _cost)) => created_outputs_for_conditions_dict(map, coin_name),
        Err(_error) => Vec::new(),
    }
}

pub fn fee_for_solution(
    puzzle_reveal: &SerializedProgram,
    solution: &SerializedProgram,
    max_cost: u64,
) -> i64 {
    match conditions_dict_for_solution(puzzle_reveal, solution, max_cost) {
        Ok((conditions, _cost)) => {
            let mut total = 0;
            match conditions.get(&ConditionOpcode::ReserveFee) {
                Some(conditions) => {
                    for cond in conditions {
                        total += atom_to_int(&cond.vars[0]);
                    }
                }
                None => {
                    total = 0;
                }
            }
            total
        }
        Err(_error) => 0,
    }
}

pub fn atom_to_int(bytes: &Vec<u8>) -> i64 {
    match bytes.len() {
        0 => 0,
        1 => i64::from_be_bytes([0, 0, 0, 0, 0, 0, 0, bytes[0]]),
        2 => i64::from_be_bytes([0, 0, 0, 0, 0, 0, bytes[0], bytes[1]]),
        4 => i64::from_be_bytes([0, 0, 0, 0, bytes[0], bytes[1], bytes[2], bytes[3]]),
        8 => i64::from_be_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ]),
        _ => 0,
    }
}

pub fn atom_to_uint(bytes: &Vec<u8>) -> u64 {
    match bytes.len() {
        0 => 0,
        1 => u64::from_be_bytes([0, 0, 0, 0, 0, 0, 0, bytes[0]]),
        2 => u64::from_be_bytes([0, 0, 0, 0, 0, 0, bytes[0], bytes[1]]),
        4 => u64::from_be_bytes([0, 0, 0, 0, bytes[0], bytes[1], bytes[2], bytes[3]]),
        8 => u64::from_be_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ]),
        _ => 0,
    }
}
