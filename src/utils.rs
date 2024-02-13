use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey, rent::Rent,
    sysvar::Sysvar,
};

/// Creates a new pda
#[inline(always)]
pub fn create_pda<'a, 'info>(
    target_account: &'a AccountInfo<'info>,
    owner: &Pubkey,
    space: usize,
    pda_seeds: &[&[u8]],
    system_program: &'a AccountInfo<'info>,
    payer: &'a AccountInfo<'info>,
) -> ProgramResult {
    let rent = Rent::get()?;
    solana_program::program::invoke_signed(
        &solana_program::system_instruction::create_account(
            payer.key,
            target_account.key,
            rent.minimum_balance(space as usize),
            space as u64,
            owner,
        ),
        &[
            payer.clone(),
            target_account.clone(),
            system_program.clone(),
        ],
        &[pda_seeds],
    )?;
    Ok(())
}
