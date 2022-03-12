#![cfg(feature = "test-bpf")]
pub mod utils;

use anchor_lang::{AccountDeserialize, AnchorDeserialize};
use mpl_auction_house::{
    config::SaleConfig, constants::AUCTION_TRADE_STATE_SIZE, receipt::ListingReceipt,
};
use mpl_testing_utils::{solana::airdrop, utils::Metadata};
use solana_program_test::*;
use solana_sdk::{clock::Clock, pubkey::Pubkey, signer::Signer};
use std::assert_eq;
use utils::setup_functions::*;

#[tokio::test]
async fn list_auction_success() {
    // **   ARRANGE **

    // Create a new ProgramTestContext with the auction house and token metadata programs.
    let mut context = auction_house_program_test().start_with_context().await;

    // Create an Auction House instance from the ProgramTestContext.
    let (ah, ahkey, _) = existing_auction_house_test_context(&mut context)
        .await
        .unwrap();

    // Set up a metadata test struct and fund wallet account which will have the ATA associated with it.
    let test_metadata = Metadata::new();
    let owner_pubkey = &test_metadata.token.pubkey();
    airdrop(&mut context, owner_pubkey, 1_000_000_000)
        .await
        .unwrap();

    // Create a new metadata account from provided data.
    test_metadata
        .create(
            &mut context,
            "Test".to_string(),
            "TST".to_string(),
            "uri".to_string(),
            None,
            100,
            false,
        )
        .await
        .unwrap();

    // Timestamp is April, 2032
    let ends_at = 1964487408;
    let min_price = 1;
    let token_size = 1;
    let sale_authority_must_sign = false;
    let high_bid_amount = 1000;
    let high_bid_trade_state = Pubkey::default();

    // Create a list_for_sale transaction.
    let ((accounts, listing_receipt_acc), list_tx) = list_for_sale(
        &mut context,
        &ahkey,
        &ah,
        &test_metadata,
        token_size,
        min_price,
        ends_at,
        sale_authority_must_sign,
        high_bid_amount,
        high_bid_trade_state,
    );

    // ** ACT **

    // Process tx w/ banks client.
    context
        .banks_client
        .process_transaction(list_tx)
        .await
        .unwrap();

    // Creation timestamp
    let timestamp = context
        .banks_client
        .get_sysvar::<Clock>()
        .await
        .unwrap()
        .unix_timestamp;

    // Get auction trade state account
    let auction_trade_state = context
        .banks_client
        .get_account(accounts.auction_trade_state)
        .await
        .expect("Error Getting Trade State")
        .expect("Trade State Empty");

    let listing_receipt_account = context
        .banks_client
        .get_account(listing_receipt_acc.receipt)
        .await
        .expect("getting listing receipt")
        .expect("empty listing receipt data");

    let listing_receipt =
        ListingReceipt::try_deserialize(&mut listing_receipt_account.data.as_ref()).unwrap();

    // ** ASSERT **

    println!("auction_trade_state.data: {:?}", &auction_trade_state.data);

    // Auction trade state data has the correct length.
    assert_eq!(auction_trade_state.data.len(), AUCTION_TRADE_STATE_SIZE);

    let sale_config: SaleConfig =
        AnchorDeserialize::deserialize(&mut auction_trade_state.data.as_ref()).unwrap();

    // Auction trade state data has the correct values.
    assert_eq!(sale_config.min_price, min_price);
    assert_eq!(sale_config.ends_at, ends_at);
    assert_eq!(
        sale_config.sale_authority_must_sign,
        sale_authority_must_sign
    );
    assert_eq!(sale_config.highest_bid.amount, high_bid_amount);
    assert_eq!(sale_config.highest_bid.trade_state, high_bid_trade_state);

    // Listing Receipt has the expected values
    assert_eq!(listing_receipt.auction_house, accounts.auction_house);
    assert_eq!(listing_receipt.metadata, accounts.metadata);
    assert_eq!(listing_receipt.seller, accounts.wallet);
    assert_eq!(listing_receipt.created_at, timestamp);
    assert_eq!(listing_receipt.purchase_receipt, None);
    assert_eq!(listing_receipt.canceled_at, None);
    assert_eq!(listing_receipt.bookkeeper, *owner_pubkey);
    assert_eq!(listing_receipt.seller, *owner_pubkey);
}
