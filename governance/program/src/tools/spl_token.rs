//! General purpose SPL token utility functions

use {
    crate::{error::GovernanceError, tools::pack::unpack_coption_pubkey},
    arrayref::array_ref,
    solana_program::{
        account_info::AccountInfo,
        entrypoint::ProgramResult,
        msg,
        program::{invoke, invoke_signed},
        program_error::ProgramError,
        program_option::COption,
        program_pack::Pack,
        pubkey::Pubkey,
        rent::Rent,
        system_instruction,
    },
    spl_token::{
        instruction::{set_authority, AuthorityType},
        state::{Account, Mint},
    }
};

/// Creates and initializes SPL token account with PDA using the provided PDA
/// seeds
#[allow(clippy::too_many_arguments)]
pub fn create_spl_token_account_signed<'a>(
    payer_info: &AccountInfo<'a>,
    token_account_info: &AccountInfo<'a>,
    token_account_address_seeds: &[&[u8]],
    token_mint_info: &AccountInfo<'a>,
    token_account_owner_info: &AccountInfo<'a>,
    program_id: &Pubkey,
    system_info: &AccountInfo<'a>,
    spl_token_info: &AccountInfo<'a>,
    rent_sysvar_info: &AccountInfo<'a>,
    rent: &Rent,
) -> Result<(), ProgramError> {
    //Define the spl_token_id to normal token
    let mut token_program_id = &spl_token::id();
    let token_2022_pubkey = spl_token_2022::id();
    //Check if the spl_token_mint owner belongs to normal token or token2022 else give an error
    if token_mint_info.owner != &spl_token::id() && token_mint_info.owner != &spl_token_2022::id(){
        return Err(GovernanceError::SplTokenMintWithInvalidOwner.into());
    }
    //Check if spl_token_mint owner belongs to token 2022 then reassign it to token 2022
    if token_mint_info.owner == &spl_token_2022::id(){
        token_program_id = &token_2022_pubkey;
    }
    let create_account_instruction = system_instruction::create_account(
        payer_info.key,
        token_account_info.key,
        1.max(rent.minimum_balance(spl_token::state::Account::get_packed_len())),
        spl_token::state::Account::get_packed_len() as u64,
        token_program_id,
    );

    let (account_address, bump_seed) =
        Pubkey::find_program_address(token_account_address_seeds, program_id);

    if account_address != *token_account_info.key {
        msg!(
            "Create SPL Token Account with PDA: {:?} was requested while PDA: {:?} was expected",
            token_account_info.key,
            account_address
        );
        return Err(ProgramError::InvalidSeeds);
    }

    let mut signers_seeds = token_account_address_seeds.to_vec();
    let bump = &[bump_seed];
    signers_seeds.push(bump);

    invoke_signed(
        &create_account_instruction,
        &[
            payer_info.clone(),
            token_account_info.clone(),
            system_info.clone(),
        ],
        &[&signers_seeds[..]],
    )?;

    let initialize_account_instruction = spl_token::instruction::initialize_account(
        token_program_id,
        token_account_info.key,
        token_mint_info.key,
        token_account_owner_info.key,
    )?;

    invoke(
        &initialize_account_instruction,
        &[
            payer_info.clone(),
            token_account_info.clone(),
            token_account_owner_info.clone(),
            token_mint_info.clone(),
            spl_token_info.clone(),
            rent_sysvar_info.clone(),
        ],
    )?;

    Ok(())
}

/// Transfers SPL Tokens
pub fn transfer_spl_tokens<'a>(
    source_info: &AccountInfo<'a>,
    destination_info: &AccountInfo<'a>,
    authority_info: &AccountInfo<'a>,
    amount: u64,
    spl_token_info: &AccountInfo<'a>,
) -> ProgramResult {
    //Define the spl_token_id to normal token
    let mut token_program_id = &spl_token::id();
    let token_2022_pubkey = spl_token_2022::id();
    //Check if the spl_token_mint owner belongs to normal token or token2022 else give an error
    if spl_token_info.key != &spl_token::id() && spl_token_info.key != &spl_token_2022::id(){
        return Err(GovernanceError::SplTokenMintWithInvalidOwner.into());
    }
    //Check if spl_token_mint owner belongs to token 2022 then reassign it to token 2022
    if spl_token_info.key == &spl_token_2022::id(){
        token_program_id = &token_2022_pubkey;
    }

    let transfer_instruction = spl_token::instruction::transfer(
        token_program_id,
        source_info.key,
        destination_info.key,
        authority_info.key,
        &[],
        amount,
    )
    .unwrap();

    invoke(
        &transfer_instruction,
        &[
            spl_token_info.clone(),
            authority_info.clone(),
            source_info.clone(),
            destination_info.clone(),
        ],
    )?;

    Ok(())
}

/// Mint SPL Tokens
pub fn mint_spl_tokens_to<'a>(
    mint_info: &AccountInfo<'a>,
    destination_info: &AccountInfo<'a>,
    mint_authority_info: &AccountInfo<'a>,
    amount: u64,
    spl_token_info: &AccountInfo<'a>,
) -> ProgramResult {
    //Define the spl_token_id to normal token
    let mut token_program_id = &spl_token::id();
    let token_2022_pubkey = spl_token_2022::id();
    //Check if the spl_token_mint owner belongs to normal token or token2022 else give an error
    if mint_info.owner != &spl_token::id() && mint_info.owner != &spl_token_2022::id(){
        return Err(GovernanceError::SplTokenMintWithInvalidOwner.into());
    }
    //Check if spl_token_mint owner belongs to token 2022 then reassign it to token 2022
    if mint_info.owner == &spl_token_2022::id(){
        token_program_id = &token_2022_pubkey;
    }
    let mint_to_ix = spl_token::instruction::mint_to(
        token_program_id,
        mint_info.key,
        destination_info.key,
        mint_authority_info.key,
        &[],
        amount,
    )
    .unwrap();

    invoke(
        &mint_to_ix,
        &[
            spl_token_info.clone(),
            mint_authority_info.clone(),
            mint_info.clone(),
            destination_info.clone(),
        ],
    )?;

    Ok(())
}

/// Transfers SPL Tokens from a token account owned by the provided PDA
/// authority with seeds
pub fn transfer_spl_tokens_signed<'a>(
    source_info: &AccountInfo<'a>,
    destination_info: &AccountInfo<'a>,
    authority_info: &AccountInfo<'a>,
    authority_seeds: &[&[u8]],
    program_id: &Pubkey,
    amount: u64,
    spl_token_info: &AccountInfo<'a>,
) -> ProgramResult {
    //Define the spl_token_id to normal token
    let mut token_program_id = &spl_token::id();
    let token_2022_pubkey = spl_token_2022::id();
    //Check if the spl_token_mint owner belongs to normal token or token2022 else give an error
    if spl_token_info.key != &spl_token::id() && spl_token_info.key != &spl_token_2022::id(){
        return Err(GovernanceError::SplTokenMintWithInvalidOwner.into());
    }
    //Check if spl_token_mint owner belongs to token 2022 then reassign it to token 2022
    if spl_token_info.key == &spl_token_2022::id(){
        token_program_id = &token_2022_pubkey;
    }

    let (authority_address, bump_seed) = Pubkey::find_program_address(authority_seeds, program_id);

    if authority_address != *authority_info.key {
        msg!(
                "Transfer SPL Token with Authority PDA: {:?} was requested while PDA: {:?} was expected",
                authority_info.key,
                authority_address
            );
        return Err(ProgramError::InvalidSeeds);
    }

    let transfer_instruction = spl_token::instruction::transfer(
        token_program_id,
        source_info.key,
        destination_info.key,
        authority_info.key,
        &[],
        amount,
    )
    .unwrap();

    let mut signers_seeds = authority_seeds.to_vec();
    let bump = &[bump_seed];
    signers_seeds.push(bump);

    invoke_signed(
        &transfer_instruction,
        &[
            spl_token_info.clone(),
            authority_info.clone(),
            source_info.clone(),
            destination_info.clone(),
        ],
        &[&signers_seeds[..]],
    )?;

    Ok(())
}

/// Burns SPL Tokens from a token account owned by the provided PDA authority
/// with seeds
pub fn burn_spl_tokens_signed<'a>(
    token_account_info: &AccountInfo<'a>,
    token_mint_info: &AccountInfo<'a>,
    authority_info: &AccountInfo<'a>,
    authority_seeds: &[&[u8]],
    program_id: &Pubkey,
    amount: u64,
    spl_token_info: &AccountInfo<'a>,
) -> ProgramResult {
    //Define the spl_token_id to normal token
    let mut token_program_id = &spl_token::id();
    let token_2022_pubkey = spl_token_2022::id();
    //Check if the spl_token_mint owner belongs to normal token or token2022 else give an error
    if token_mint_info.owner != &spl_token::id() && token_mint_info.owner != &spl_token_2022::id(){
        return Err(GovernanceError::SplTokenMintWithInvalidOwner.into());
    }
    //Check if spl_token_mint owner belongs to token 2022 then reassign it to token 2022
    if token_mint_info.owner == &spl_token_2022::id(){
        token_program_id = &token_2022_pubkey;
    }
    let (authority_address, bump_seed) = Pubkey::find_program_address(authority_seeds, program_id);

    if authority_address != *authority_info.key {
        msg!(
            "Burn SPL Token with Authority PDA: {:?} was requested while PDA: {:?} was expected",
            authority_info.key,
            authority_address
        );
        return Err(ProgramError::InvalidSeeds);
    }

    let burn_ix = spl_token::instruction::burn(
        token_program_id,
        token_account_info.key,
        token_mint_info.key,
        authority_info.key,
        &[],
        amount,
    )
    .unwrap();

    let mut signers_seeds = authority_seeds.to_vec();
    let bump = &[bump_seed];
    signers_seeds.push(bump);

    invoke_signed(
        &burn_ix,
        &[
            spl_token_info.clone(),
            token_account_info.clone(),
            token_mint_info.clone(),
            authority_info.clone(),
        ],
        &[&signers_seeds[..]],
    )?;

    Ok(())
}

/// Asserts the given account_info represents a valid SPL Token account which is
/// initialized and belongs to spl_token program
pub fn assert_is_valid_spl_token_account(account_info: &AccountInfo) -> Result<(), ProgramError> {
    if account_info.data_is_empty() {
        return Err(GovernanceError::SplTokenAccountDoesNotExist.into());
    }

    if account_info.owner != &spl_token::id() && account_info.owner != &spl_token_2022::id() {
        return Err(GovernanceError::SplTokenAccountWithInvalidOwner.into());
    }

    if account_info.data_len() != Account::LEN {
        return Err(GovernanceError::SplTokenInvalidTokenAccountData.into());
    }

    // TokenAccount layout:
    //  mint(32)
    //  owner(32)
    //  amount(8)
    //  delegate(36)
    //  state(1)
    //  ...
    let data = account_info.try_borrow_data()?;
    let state = array_ref![data, 108, 1];

    if state == &[0] {
        return Err(GovernanceError::SplTokenAccountNotInitialized.into());
    }

    Ok(())
}

/// Checks if the given account_info  is spl-token token account
pub fn is_spl_token_account(account_info: &AccountInfo) -> bool {
    assert_is_valid_spl_token_account(account_info).is_ok()
}

/// Asserts the given mint_info represents a valid SPL Token Mint account  which
/// is initialized and belongs to spl_token program
pub fn assert_is_valid_spl_token_mint(mint_info: &AccountInfo) -> Result<(), ProgramError> {
    if mint_info.data_is_empty() {
        return Err(GovernanceError::SplTokenMintDoesNotExist.into());
    }

    if mint_info.owner != &spl_token::id() && mint_info.owner != &spl_token_2022::id() {
        return Err(GovernanceError::SplTokenMintWithInvalidOwner.into());
    }

    if mint_info.data_len() != Mint::LEN {
        return Err(GovernanceError::SplTokenInvalidMintAccountData.into());
    }

    // In token program [36, 8, 1, is_initialized(1), 36] is the layout
    let data = mint_info.try_borrow_data()?;
    let is_initialized = array_ref![data, 45, 1];

    if is_initialized == &[0] {
        return Err(GovernanceError::SplTokenMintNotInitialized.into());
    }

    Ok(())
}

/// Checks if the given account_info is be spl-token mint account
pub fn is_spl_token_mint(mint_info: &AccountInfo) -> bool {
    assert_is_valid_spl_token_mint(mint_info).is_ok()
}

/// Computationally cheap method to get mint from a token account
/// It reads mint without deserializing full account data
pub fn get_spl_token_mint(token_account_info: &AccountInfo) -> Result<Pubkey, ProgramError> {
    assert_is_valid_spl_token_account(token_account_info)?;

    // TokeAccount layout:   mint(32), owner(32), amount(8), ...
    let data = token_account_info.try_borrow_data()?;
    let mint_data = array_ref![data, 0, 32];
    Ok(Pubkey::new_from_array(*mint_data))
}

/// Computationally cheap method to get owner from a token account
/// It reads owner without deserializing full account data
pub fn get_spl_token_owner(token_account_info: &AccountInfo) -> Result<Pubkey, ProgramError> {
    assert_is_valid_spl_token_account(token_account_info)?;

    // TokeAccount layout:   mint(32), owner(32), amount(8)
    let data = token_account_info.try_borrow_data()?;
    let owner_data = array_ref![data, 32, 32];
    Ok(Pubkey::new_from_array(*owner_data))
}

/// Computationally cheap method to just get supply from a mint without
/// unpacking the whole object
pub fn get_spl_token_mint_supply(mint_info: &AccountInfo) -> Result<u64, ProgramError> {
    assert_is_valid_spl_token_mint(mint_info)?;
    // In token program, 36, 8, 1, 1 is the layout, where the first 8 is supply u64.
    // so we start at 36.
    let data = mint_info.try_borrow_data().unwrap();
    let bytes = array_ref![data, 36, 8];

    Ok(u64::from_le_bytes(*bytes))
}

/// Computationally cheap method to just get authority from a mint without
/// unpacking the whole object
pub fn get_spl_token_mint_authority(
    mint_info: &AccountInfo,
) -> Result<COption<Pubkey>, ProgramError> {
    assert_is_valid_spl_token_mint(mint_info)?;
    // In token program, 36, 8, 1, 1 is the layout, where the first 36 is authority.
    let data = mint_info.try_borrow_data().unwrap();
    let bytes = array_ref![data, 0, 36];

    unpack_coption_pubkey(bytes)
}

/// Asserts current mint authority matches the given authority and it's signer
/// of the transaction
pub fn assert_spl_token_mint_authority_is_signer(
    mint_info: &AccountInfo,
    mint_authority_info: &AccountInfo,
) -> Result<(), ProgramError> {
    let mint_authority = get_spl_token_mint_authority(mint_info)?;

    if mint_authority.is_none() {
        return Err(GovernanceError::MintHasNoAuthority.into());
    }

    if !mint_authority.contains(mint_authority_info.key) {
        return Err(GovernanceError::InvalidMintAuthority.into());
    }

    if !mint_authority_info.is_signer {
        return Err(GovernanceError::MintAuthorityMustSign.into());
    }

    Ok(())
}

/// Asserts current token owner matches the given owner and it's signer of the
/// transaction
pub fn assert_spl_token_owner_is_signer(
    token_info: &AccountInfo,
    token_owner_info: &AccountInfo,
) -> Result<(), ProgramError> {
    let token_owner = get_spl_token_owner(token_info)?;

    if token_owner != *token_owner_info.key {
        return Err(GovernanceError::InvalidTokenOwner.into());
    }

    if !token_owner_info.is_signer {
        return Err(GovernanceError::TokenOwnerMustSign.into());
    }

    Ok(())
}

/// Sets spl-token account (Mint or TokenAccount) authority
pub fn set_spl_token_account_authority<'a>(
    account_info: &AccountInfo<'a>,
    account_authority: &AccountInfo<'a>,
    new_account_authority: &Pubkey,
    authority_type: AuthorityType,
    spl_token_info: &AccountInfo<'a>,
) -> Result<(), ProgramError> {
    //Define the spl_token_id to normal token
    let mut token_program_id = &spl_token::id();
    let token_2022_pubkey = spl_token_2022::id();
    //Check if the spl_token_mint owner belongs to normal token or token2022 else give an error
    if spl_token_info.key != &spl_token::id() && spl_token_info.key != &spl_token_2022::id(){
        return Err(GovernanceError::SplTokenMintWithInvalidOwner.into());
    }
    //Check if spl_token_mint owner belongs to token 2022 then reassign it to token 2022
    if spl_token_info.key == &spl_token_2022::id(){ 
        token_program_id = &token_2022_pubkey;
    }
    let set_authority_ix = set_authority(
        token_program_id,
        account_info.key,
        Some(new_account_authority),
        authority_type,
        account_authority.key,
        &[],
    )?;

    invoke(
        &set_authority_ix,
        &[
            account_info.clone(),
            account_authority.clone(),
            spl_token_info.clone(),
        ],
    )?;

    Ok(())
}
