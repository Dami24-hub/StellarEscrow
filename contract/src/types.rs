use soroban_sdk::{contracttype, Address, String, Vec};

/// Maximum byte length for a single metadata value string
pub const METADATA_MAX_VALUE_LEN: u32 = 256;
/// Maximum number of key-value pairs in metadata
pub const METADATA_MAX_ENTRIES: u32 = 10;

// ---------------------------------------------------------------------------
// Fee Tier System
// ---------------------------------------------------------------------------

pub const TIER_SILVER_THRESHOLD: u64 = 10_000_000_000;
pub const TIER_GOLD_THRESHOLD: u64 = 100_000_000_000;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UserTier {
    Bronze,
    Silver,
    Gold,
    Custom,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UserTierInfo {
    pub tier: UserTier,
    pub total_volume: u64,
    pub custom_fee_bps: Option<u32>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TierConfig {
    pub bronze_fee_bps: u32,
    pub silver_fee_bps: u32,
    pub gold_fee_bps: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TradeStatus {
    Created,
    Funded,
    Completed,
    Disputed,
    Cancelled,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DisputeResolution {
    ReleaseToBuyer,
    ReleaseToSeller,
}

/// A single metadata key-value entry
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MetadataEntry {
    pub key: String,
    pub value: String,
}

/// Structured metadata attached to a trade (e.g. product description, shipping info)
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TradeMetadata {
    pub entries: Vec<MetadataEntry>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Trade {
    pub id: u64,
    pub seller: Address,
    pub buyer: Address,
    pub amount: u64,
    pub fee: u64,
    pub arbitrator: Option<Address>,
    pub status: TradeStatus,
    /// Optional structured metadata (product info, shipping details, etc.)
    pub metadata: Option<TradeMetadata>,
}

// ---------------------------------------------------------------------------
// Subscription Model
// ---------------------------------------------------------------------------

/// Duration of a subscription in ledgers (~1 ledger ≈ 5 s; 30 days ≈ 518_400 ledgers)
pub const SUBSCRIPTION_DURATION_LEDGERS: u32 = 518_400;

/// Monthly price in stroops (USDC micro-units) per tier
pub const SUB_PRICE_BASIC: u64 = 5_000_000;   // 5 USDC
pub const SUB_PRICE_PRO: u64 = 15_000_000;    // 15 USDC
pub const SUB_PRICE_ENTERPRISE: u64 = 50_000_000; // 50 USDC

/// Fee discounts in bps applied on top of the tier/base fee
pub const SUB_DISCOUNT_BASIC_BPS: u32 = 20;       // −0.20 %
pub const SUB_DISCOUNT_PRO_BPS: u32 = 50;          // −0.50 %
pub const SUB_DISCOUNT_ENTERPRISE_BPS: u32 = 100;  // −1.00 %

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SubscriptionTier {
    Basic,
    Pro,
    Enterprise,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Subscription {
    pub subscriber: Address,
    pub tier: SubscriptionTier,
    /// Ledger sequence at which the subscription expires
    pub expires_at: u32,
    /// Ledger sequence of the last renewal / purchase
    pub renewed_at: u32,
}

// ---------------------------------------------------------------------------
// Trade Templates
// ---------------------------------------------------------------------------

pub const TEMPLATE_NAME_MAX_LEN: u32 = 64;
pub const TEMPLATE_MAX_VERSIONS: u32 = 10;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TemplateTerms {
    pub description: String,
    pub default_arbitrator: Option<Address>,
    pub fixed_amount: Option<u64>,
    pub default_metadata: Option<TradeMetadata>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TemplateVersion {
    pub version: u32,
    pub terms: TemplateTerms,
    pub created_at: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TradeTemplate {
    pub id: u64,
    pub owner: Address,
    pub name: String,
    pub current_version: u32,
    pub versions: Vec<TemplateVersion>,
    pub active: bool,
    pub created_at: u32,
    pub updated_at: u32,
}
