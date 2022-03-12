use anchor_lang::{prelude::*, AnchorDeserialize, AnchorSerialize};

#[account]
pub struct AuctionConfig {
    pub track_highest_bid: bool,
    restrict_high_bidder_cancelation: bool,
    can_cancel_timed_auction: bool,
    approved_sale_authorities: Vec<Pubkey>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct SaleConfig {
    pub min_price: u64,
    pub ends_at: u32,
    pub sale_authority_must_sign: bool,
    pub highest_bid: Bid,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct Bid {
    pub amount: u64,
    pub trade_state: Pubkey,
}
