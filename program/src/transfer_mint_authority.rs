use ore_api::prelude::*;
use solana_program::pubkey;
use spl_token_2022::instruction::AuthorityType;
use steel::*;

const NEW_TREASURY_ADDRESS: Pubkey = pubkey!("45db2FSR4mcXdSVVZbKbwojU6uYDpMyhpEi7cC8nHaWG");

/// TransferMintAuthority transfers the mint authority to a new address.
pub fn process_transfer_mint_authority(
    accounts: &[AccountInfo<'_>],
    _data: &[u8],
) -> ProgramResult {
    // Load accounts.
    let [signer_info, mint_info, treasury_info, new_treasury_info, token_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info
        .is_signer()?
        .has_address(&ore_api::consts::INITIALIZER_ADDRESS)?;
    mint_info
        .has_address(&ore_api::consts::MINT_ADDRESS)?
        .is_writable()?;
    treasury_info.is_treasury()?.is_writable()?;
    new_treasury_info.has_address(&NEW_TREASURY_ADDRESS)?;
    token_program.is_program(&spl_token::ID)?;

    // Transfer the mint authority to the new treasury.
    set_authority_signed(
        mint_info,
        treasury_info,
        Some(new_treasury_info),
        AuthorityType::MintTokens,
        token_program,
        &[TREASURY],
    )?;

    Ok(())
}
