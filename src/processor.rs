use {
    crate::{id, instruction::SkeletonInstruction},
    borsh::BorshDeserialize,
    solana_program::{
        account_info::{next_account_info, AccountInfo},
        entrypoint::ProgramResult,
        msg,
        program_error::ProgramError,
        pubkey::Pubkey,
    },
};

pub struct Processor {}
impl Processor {
    fn process_do_something(accounts: &[AccountInfo]) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let _some_account_info = next_account_info(account_info_iter)?;

        // do stuff

        Ok(())
    }

    /// Processes [Instruction](enum.Instruction.html).
    pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], input: &[u8]) -> ProgramResult {
        if *program_id != id() {
            return Err(ProgramError::IncorrectProgramId);
        }

        let instruction = SkeletonInstruction::try_from_slice(input)?;

        match instruction {
            SkeletonInstruction::DoSomething => {
                msg!("Instruction: DoSomething");
                Self::process_do_something(accounts)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_something() {
        assert_eq!(id(), id());
    }
}
