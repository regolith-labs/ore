use solana_program::pubkey;
use steel::*;

use spl_token_2022::{
    extension::{
        metadata_pointer::MetadataPointer, BaseStateWithExtensions, PodStateWithExtensions,
    },
    pod::{PodCOption, PodMint},
};

/// Claims ORE for seeker device.
pub fn process_claim_seeker(accounts: &[AccountInfo<'_>], _data: &[u8]) -> ProgramResult {
    // Load accounts.
    let [signer_info, mint_info, _token_info] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    mint_info.has_owner(&spl_token_2022::ID)?;

    // Load mint.
    let mint_data = mint_info.try_borrow_data()?;
    let mint = PodStateWithExtensions::<PodMint>::unpack(&mint_data)?;

    // Check mint authority.
    assert!(
        mint.base.mint_authority
            == PodCOption::some(pubkey!("GT2zuHVaZQYZSyQMgJPLzvkmyztfyXg2NJunqFp4p3A4")),
        "mint authority mismatch"
    );

    // Check metadata pointer.
    let ext = mint.get_extension::<MetadataPointer>()?;
    assert!(
        ext.authority.0 == pubkey!("GT2zuHVaZQYZSyQMgJPLzvkmyztfyXg2NJunqFp4p3A4"),
        "metadata authority mismatch"
    );
    assert!(
        ext.metadata_address.0 == pubkey!("GT22s89nU4iWFkNXj1Bw6uYhJJWDRPpShHt4Bk8f99Te"),
        "metadata address mismatch"
    );

    Ok(())
}
