use soroban_sdk::{contracttype, Address, String, Vec};

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

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SortOrder {
    Ascending,
    Descending,
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
    /// Ledger sequence number when the trade was created
    pub created_at: u32,
    /// Ledger sequence number of the last status update
    pub updated_at: u32,
    /// Optional structured metadata (product info, shipping details, etc.)
    pub metadata: Option<TradeMetadata>,
}

/// A richer view of a trade used for history queries
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransactionRecord {
    pub trade_id: u64,
    pub seller: Address,
    pub buyer: Address,
    pub amount: u64,
    pub fee: u64,
    pub status: TradeStatus,
    pub created_at: u32,
    pub updated_at: u32,
    pub metadata: Option<TradeMetadata>,
}

/// Maximum byte length for a single metadata value string
pub const METADATA_MAX_VALUE_LEN: u32 = 256;
/// Maximum number of key-value pairs in metadata
pub const METADATA_MAX_ENTRIES: u32 = 10;

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

// ---------------------------------------------------------------------------
// Fee Tier System
// ---------------------------------------------------------------------------

/// Volume thresholds (in USDC micro-units) for automatic tier upgrades.
/// Bronze: 0+, Silver: 10_000_000_000 (10k USDC), Gold: 100_000_000_000 (100k USDC)
pub const TIER_SILVER_THRESHOLD: u64 = 10_000_000_000;
pub const TIER_GOLD_THRESHOLD: u64 = 100_000_000_000;

/// User membership tier — determines the fee rate applied to their trades.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UserTier {
    /// Default tier — uses the platform base fee
    Bronze,
    /// Mid tier — reduced fee rate
    Silver,
    /// Top tier — lowest fee rate
    Gold,
    /// Manually assigned custom fee rate (overrides volume-based tier)
    Custom,
}

/// Per-user tier record stored on-chain.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UserTierInfo {
    /// Current tier
    pub tier: UserTier,
    /// Cumulative completed trade volume (sum of trade amounts)
    pub total_volume: u64,
    /// Custom fee in basis points — only used when tier == Custom
    pub custom_fee_bps: Option<u32>,
}

/// Tier configuration set by admin — defines fee bps per tier.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TierConfig {
    /// Fee bps for Bronze (default: platform base fee)
    pub bronze_fee_bps: u32,
    /// Fee bps for Silver
    pub silver_fee_bps: u32,
    /// Fee bps for Gold
    pub gold_fee_bps: u32,
}

/// Filter options for history queries
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HistoryFilter {
    /// Optional status filter
    pub status: Option<TradeStatus>,
    /// Minimum ledger sequence (inclusive)
    pub from_ledger: Option<u32>,
    /// Maximum ledger sequence (inclusive)
    pub to_ledger: Option<u32>,
}

/// Paginated history result
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HistoryPage {
    pub records: Vec<TransactionRecord>,
    pub total: u32,
    pub offset: u32,
    pub limit: u32,
}

// ---------------------------------------------------------------------------
// Trade Templates
// ---------------------------------------------------------------------------

/// Maximum length of a template name string
pub const TEMPLATE_NAME_MAX_LEN: u32 = 64;
/// Maximum number of active versions retained per template
pub const TEMPLATE_MAX_VERSIONS: u32 = 10;

/// Predefined terms attached to a template
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TemplateTerms {
    /// Human-readable description of the trade terms
    pub description: String,
    /// Optional default arbitrator for trades from this template
    pub default_arbitrator: Option<Address>,
    /// Optional fixed amount — if set, trades must use this amount
    pub fixed_amount: Option<u64>,
    /// Optional metadata defaults applied to every trade from this template
    pub default_metadata: Option<TradeMetadata>,
}

/// A single versioned snapshot of a template
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TemplateVersion {
    pub version: u32,
    pub terms: TemplateTerms,
    /// Ledger sequence when this version was created
    pub created_at: u32,
}

/// A reusable trade template owned by a seller
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TradeTemplate {
    pub id: u64,
    pub owner: Address,
    pub name: String,
    /// Current (latest) version number
    pub current_version: u32,
    /// All retained versions (newest last)
    pub versions: Vec<TemplateVersion>,
    /// Whether the template is active and can be used to create trades
    pub active: bool,
    pub created_at: u32,
    pub updated_at: u32,
}
