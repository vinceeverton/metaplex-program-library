pub const PREFIX: &str = "auction_house";
pub const AUCTION: &str = "auction";
pub const FEE_PAYER: &str = "fee_payer";
pub const TREASURY: &str = "treasury";
pub const SIGNER: &str = "signer";
pub const PURCHASE_RECEIPT_PREFIX: &str = "purchase_receipt";
pub const BID_RECEIPT_PREFIX: &str = "bid_receipt";
pub const LISTING_RECEIPT_PREFIX: &str = "listing_receipt";

// 8 - account discriminator
// 8 - min_price: u64
// 4 - ends_at: u32
// 1 - sale_authority_must_sign: bool
// 8 - bid amount: u64
// 32 - trade_state: Pubkey
pub const AUCTION_TRADE_STATE_SIZE: usize = 61;

pub const PUBLIC_BUY_SIGHASH: [u8; 8] = [169, 84, 218, 35, 42, 206, 16, 171];
pub const PRIVATE_BUY_SIGHASH: [u8; 8] = [102, 6, 61, 18, 1, 218, 235, 234];
pub const INSTANT_SELL_SIGHASH: [u8; 8] = [51, 230, 133, 164, 1, 127, 131, 173];
pub const CREATE_AUCTION_LISTING_SIGHASH: [u8; 8] = [104, 106, 53, 140, 163, 107, 143, 158];
