use crate::{config::*, constants::*, utils::*, AuctionHouse, ErrorCode};
use anchor_lang::{
    prelude::*, solana_program::program::invoke, AnchorDeserialize, AnchorSerialize,
};
use anchor_spl::token::{Token, TokenAccount};
use spl_token::instruction::approve;

// #[derive(Accounts)]
// pub struct Bid<'info> {}

// #[derive(Accounts)]
// pub struct CancelListing<'info> {}

// #[derive(Accounts)]
// pub struct CancelBid<'info> {}

// #[derive(Accounts)]
// pub struct ExecuteAuctionSale<'info> {}

#[derive(Accounts)]
#[instruction(auction_trade_state_bump: u8, program_as_signer_bump: u8, token_size: u64)]
pub struct ListForSale<'info> {
    pub wallet: UncheckedAccount<'info>,
    #[account(mut)]
    token_account: Account<'info, TokenAccount>,
    metadata: UncheckedAccount<'info>,
    authority: UncheckedAccount<'info>,
    #[account(seeds=[PREFIX.as_bytes(), auction_house.creator.as_ref(), auction_house.treasury_mint.as_ref()], bump=auction_house.bump, has_one=authority, has_one=auction_house_fee_account)]
    auction_house: Account<'info, AuctionHouse>,
    #[account(mut, seeds=[PREFIX.as_bytes(), auction_house.key().as_ref(), FEE_PAYER.as_bytes()], bump=auction_house.fee_payer_bump)]
    auction_house_fee_account: UncheckedAccount<'info>,
    #[account(mut, seeds=[PREFIX.as_bytes(), AUCTION.as_bytes(), wallet.key().as_ref(), auction_house.key().as_ref(), token_account.key().as_ref(), auction_house.treasury_mint.as_ref(), token_account.mint.as_ref(), &token_size.to_le_bytes()], bump=auction_trade_state_bump)]
    auction_trade_state: UncheckedAccount<'info>,
    token_program: Program<'info, Token>,
    system_program: Program<'info, System>,
    #[account(seeds=[PREFIX.as_bytes(), SIGNER.as_bytes()], bump=program_as_signer_bump)]
    program_as_signer: UncheckedAccount<'info>,
    rent: Sysvar<'info, Rent>,
}

pub fn list_for_sale(
    ctx: Context<ListForSale>,
    auction_trade_state_bump: u8,
    _program_as_signer_bump: u8,
    token_size: u64,
    min_price: u64,
    ends_at: u32,
    sale_authority_must_sign: bool,
    high_bid_amount: u64,
    high_bid_trade_state: Pubkey,
) -> ProgramResult {
    let wallet = &ctx.accounts.wallet;
    let token_account = &ctx.accounts.token_account;
    let metadata = &ctx.accounts.metadata;
    let authority = &ctx.accounts.authority;
    let auction_trade_state = &ctx.accounts.auction_trade_state;
    let auction_house = &ctx.accounts.auction_house;
    let auction_house_fee_account = &ctx.accounts.auction_house_fee_account;
    let token_program = &ctx.accounts.token_program;
    let system_program = &ctx.accounts.system_program;
    let program_as_signer = &ctx.accounts.program_as_signer;
    let rent = &ctx.accounts.rent;

    // Wallet must be signer to list item for sale.
    if !wallet.to_account_info().is_signer {
        return Err(ErrorCode::SaleRequiresSigner.into());
    }

    let auction_house_key = auction_house.key();

    // Seeds for the auction_house_fee account.
    let seeds = [
        PREFIX.as_bytes(),
        auction_house_key.as_ref(),
        FEE_PAYER.as_bytes(),
        &[auction_house.fee_payer_bump],
    ];

    // Determine whether the Auction House or wallet should be the fee payer.
    // If authority is signer, set AH to be fee payer, otherwise, the wallet.
    let (fee_payer, fee_seeds) = get_fee_payer(
        authority,
        auction_house,
        wallet.to_account_info(),
        auction_house_fee_account.to_account_info(),
        &seeds,
    )?;

    // Ensure the passed in token account is actually an associated token account for the wallet.
    assert_is_ata(
        &token_account.to_account_info(),
        &wallet.key(),
        &token_account.mint,
    )?;

    // Ensure the metadata account is derived from the token account's mint.
    assert_metadata_valid(metadata, token_account)?;

    // Ensure the token account has enough tokens in it
    if token_size > token_account.amount {
        return Err(ErrorCode::InvalidTokenAmount.into());
    }

    // If the wallet is the signer, approve Auction House as the delegated authority for the token account.
    if wallet.is_signer {
        invoke(
            &approve(
                &token_program.key(),
                &token_account.key(),
                &program_as_signer.key(),
                &wallet.key(),
                &[],
                token_size,
            )
            .unwrap(),
            &[
                token_program.to_account_info(),
                token_account.to_account_info(),
                program_as_signer.to_account_info(),
                wallet.to_account_info(),
            ],
        )?;
    }

    // If auction trade state doesn't exist, create the account.
    let ts_info = auction_trade_state.to_account_info();
    if ts_info.data_is_empty() {
        let token_account_key = token_account.key();
        let wallet_key = wallet.key();
        let ts_seeds = [
            PREFIX.as_bytes(),
            AUCTION.as_bytes(),
            wallet_key.as_ref(),
            auction_house_key.as_ref(),
            token_account_key.as_ref(),
            auction_house.treasury_mint.as_ref(),
            token_account.mint.as_ref(),
            &token_size.to_le_bytes(),
            &[auction_trade_state_bump],
        ];
        create_or_allocate_account_raw(
            *ctx.program_id,
            &ts_info,
            &rent.to_account_info(),
            &system_program,
            &fee_payer,
            AUCTION_TRADE_STATE_SIZE,
            fee_seeds,
            &ts_seeds,
        )?;
    }

    let highest_bid = Bid {
        amount: high_bid_amount,
        trade_state: high_bid_trade_state,
    };

    let sale_config = SaleConfig {
        min_price,
        ends_at,
        sale_authority_must_sign,
        highest_bid,
    };

    let mut data = ts_info.data.borrow_mut();
    AnchorSerialize::serialize(&sale_config, &mut *data)?;

    Ok(())
}
