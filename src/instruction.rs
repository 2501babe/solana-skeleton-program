//! Instruction types

#![allow(clippy::too_many_arguments)]

use {
    borsh::{BorshDeserialize, BorshSerialize},
    solana_program::{
        instruction::{AccountMeta, Instruction},
        pubkey::Pubkey,
    },
};

#[repr(C)]
#[derive(Clone, Debug, PartialEq, BorshSerialize, BorshDeserialize)]
pub enum SkeletonInstruction {
    ///   Do something
    ///
    ///   0. `[]` Some account
    DoSomething,
}

/// Creates a `DoSomething` instruction.
pub fn do_something(program_id: &Pubkey, some_account: &Pubkey) -> Instruction {
    let data = borsh::to_vec(&SkeletonInstruction::DoSomething).unwrap();
    let accounts = vec![AccountMeta::new_readonly(*some_account, false)];

    Instruction {
        program_id: *program_id,
        accounts,
        data,
    }
}
