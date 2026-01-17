//! Margin Trading API response models.
//!
//! Models for the Binance Margin SAPI endpoints.

use serde::{Deserialize, Serialize};

use super::string_or_float;
use crate::types::{OrderSide, OrderStatus, OrderType, TimeInForce};

/// Margin transfer type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MarginTransferType {
    /// Transfer from main account to margin account
    #[serde(rename = "1")]
    MainToMargin,
    /// Transfer from margin account to main account
    #[serde(rename = "2")]
    MarginToMain,
}

/// Isolated margin transfer type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum IsolatedMarginTransferType {
    /// Spot account
    Spot,
    /// Isolated margin account
    IsolatedMargin,
}

/// Side effect type for margin orders.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SideEffectType {
    /// No side effect
    #[default]
    NoSideEffect,
    /// Margin buy
    MarginBuy,
    /// Auto repay
    AutoRepay,
}

/// Transaction ID response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionId {
    /// Transaction ID
    pub tran_id: u64,
}

/// Margin account details.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarginAccountDetails {
    /// Whether borrow is enabled.
    pub borrow_enabled: bool,
    /// Margin level.
    #[serde(with = "string_or_float")]
    pub margin_level: f64,
    /// Total asset in BTC.
    #[serde(with = "string_or_float")]
    pub total_asset_of_btc: f64,
    /// Total liability in BTC.
    #[serde(with = "string_or_float")]
    pub total_liability_of_btc: f64,
    /// Total net asset in BTC.
    #[serde(with = "string_or_float")]
    pub total_net_asset_of_btc: f64,
    /// Whether trading is enabled.
    pub trade_enabled: bool,
    /// Whether transfer is enabled.
    pub transfer_enabled: bool,
    /// User assets.
    pub user_assets: Vec<MarginAsset>,
}

/// Margin asset balance.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarginAsset {
    /// Asset symbol.
    pub asset: String,
    /// Borrowed amount.
    #[serde(with = "string_or_float")]
    pub borrowed: f64,
    /// Free amount.
    #[serde(with = "string_or_float")]
    pub free: f64,
    /// Interest amount.
    #[serde(with = "string_or_float")]
    pub interest: f64,
    /// Locked amount.
    #[serde(with = "string_or_float")]
    pub locked: f64,
    /// Net asset amount.
    #[serde(with = "string_or_float")]
    pub net_asset: f64,
}

/// Isolated margin account details.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IsolatedMarginAccountDetails {
    /// Assets.
    #[serde(default)]
    pub assets: Vec<IsolatedMarginAccountAsset>,
    /// Total asset in BTC (optional).
    #[serde(default, with = "string_or_float_option")]
    pub total_asset_of_btc: Option<f64>,
    /// Total liability in BTC (optional).
    #[serde(default, with = "string_or_float_option")]
    pub total_liability_of_btc: Option<f64>,
    /// Total net asset in BTC (optional).
    #[serde(default, with = "string_or_float_option")]
    pub total_net_asset_of_btc: Option<f64>,
}

/// Isolated margin account asset.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IsolatedMarginAccountAsset {
    /// Base asset.
    pub base_asset: IsolatedAssetDetails,
    /// Quote asset.
    pub quote_asset: IsolatedAssetDetails,
    /// Symbol.
    pub symbol: String,
    /// Whether isolated trading is created.
    pub isolated_created: bool,
    /// Whether margin trading is enabled.
    pub enabled: bool,
    /// Margin level.
    #[serde(with = "string_or_float")]
    pub margin_level: f64,
    /// Margin ratio.
    #[serde(with = "string_or_float")]
    pub margin_ratio: f64,
    /// Index price.
    #[serde(with = "string_or_float")]
    pub index_price: f64,
    /// Liquidate price.
    #[serde(with = "string_or_float")]
    pub liquidate_price: f64,
    /// Liquidate rate.
    #[serde(with = "string_or_float")]
    pub liquidate_rate: f64,
    /// Whether trading is enabled.
    pub trade_enabled: bool,
}

/// Isolated asset details.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IsolatedAssetDetails {
    /// Asset symbol.
    pub asset: String,
    /// Whether borrow is enabled.
    pub borrow_enabled: bool,
    /// Borrowed amount.
    #[serde(with = "string_or_float")]
    pub borrowed: f64,
    /// Free amount.
    #[serde(with = "string_or_float")]
    pub free: f64,
    /// Interest amount.
    #[serde(with = "string_or_float")]
    pub interest: f64,
    /// Locked amount.
    #[serde(with = "string_or_float")]
    pub locked: f64,
    /// Net asset amount.
    #[serde(with = "string_or_float")]
    pub net_asset: f64,
    /// Net asset in BTC.
    #[serde(with = "string_or_float")]
    pub net_asset_of_btc: f64,
    /// Whether repay is enabled.
    pub repay_enabled: bool,
    /// Total asset amount.
    #[serde(with = "string_or_float")]
    pub total_asset: f64,
}

/// Max borrowable amount.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MaxBorrowableAmount {
    /// Amount.
    #[serde(with = "string_or_float")]
    pub amount: f64,
    /// Borrow limit.
    #[serde(default, with = "string_or_float_option")]
    pub borrow_limit: Option<f64>,
}

/// Max transferable amount.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MaxTransferableAmount {
    /// Amount.
    #[serde(with = "string_or_float")]
    pub amount: f64,
}

/// Margin order result.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarginOrderResult {
    /// Symbol.
    pub symbol: String,
    /// Order ID.
    pub order_id: u64,
    /// Client order ID.
    pub client_order_id: String,
    /// Transaction time.
    pub transact_time: u64,
    /// Price.
    #[serde(with = "string_or_float")]
    pub price: f64,
    /// Original quantity.
    #[serde(with = "string_or_float")]
    pub orig_qty: f64,
    /// Executed quantity.
    #[serde(with = "string_or_float")]
    pub executed_qty: f64,
    /// Cumulative quote quantity.
    #[serde(with = "string_or_float")]
    pub cummulative_quote_qty: f64,
    /// Order status.
    pub status: OrderStatus,
    /// Time in force.
    pub time_in_force: TimeInForce,
    /// Order type.
    #[serde(rename = "type")]
    pub order_type: OrderType,
    /// Order side.
    pub side: OrderSide,
    /// Whether this is isolated margin.
    #[serde(default)]
    pub is_isolated: Option<bool>,
}

/// Margin order state (for query).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarginOrderState {
    /// Client order ID.
    pub client_order_id: String,
    /// Cumulative quote quantity.
    #[serde(with = "string_or_float")]
    pub cummulative_quote_qty: f64,
    /// Executed quantity.
    #[serde(with = "string_or_float")]
    pub executed_qty: f64,
    /// Iceberg quantity.
    #[serde(default, with = "string_or_float_option")]
    pub iceberg_qty: Option<f64>,
    /// Whether working.
    pub is_working: bool,
    /// Order ID.
    pub order_id: u64,
    /// Original quantity.
    #[serde(with = "string_or_float")]
    pub orig_qty: f64,
    /// Price.
    #[serde(with = "string_or_float")]
    pub price: f64,
    /// Order side.
    pub side: OrderSide,
    /// Order status.
    pub status: OrderStatus,
    /// Stop price.
    #[serde(default, with = "string_or_float_option")]
    pub stop_price: Option<f64>,
    /// Symbol.
    pub symbol: String,
    /// Whether this is isolated margin.
    #[serde(default)]
    pub is_isolated: Option<bool>,
    /// Time.
    pub time: u64,
    /// Time in force.
    pub time_in_force: TimeInForce,
    /// Order type.
    #[serde(rename = "type")]
    pub order_type: OrderType,
    /// Update time.
    pub update_time: u64,
}

/// Margin order cancellation result.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarginOrderCancellation {
    /// Symbol.
    pub symbol: String,
    /// Order ID.
    #[serde(default)]
    pub order_id: Option<u64>,
    /// Original client order ID.
    pub orig_client_order_id: String,
    /// Client order ID.
    pub client_order_id: String,
    /// Price.
    #[serde(with = "string_or_float")]
    pub price: f64,
    /// Original quantity.
    #[serde(with = "string_or_float")]
    pub orig_qty: f64,
    /// Executed quantity.
    #[serde(with = "string_or_float")]
    pub executed_qty: f64,
    /// Cumulative quote quantity.
    #[serde(with = "string_or_float")]
    pub cummulative_quote_qty: f64,
    /// Order status.
    pub status: OrderStatus,
    /// Time in force.
    pub time_in_force: TimeInForce,
    /// Order type.
    #[serde(rename = "type")]
    pub order_type: OrderType,
    /// Order side.
    pub side: OrderSide,
}

/// Margin trade record.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarginTrade {
    /// Commission.
    #[serde(with = "string_or_float")]
    pub commission: f64,
    /// Commission asset.
    pub commission_asset: String,
    /// Trade ID.
    pub id: u64,
    /// Whether buyer.
    pub is_buyer: bool,
    /// Whether isolated margin.
    #[serde(default)]
    pub is_isolated: Option<bool>,
    /// Whether maker.
    pub is_maker: bool,
    /// Order ID.
    pub order_id: u64,
    /// Price.
    #[serde(with = "string_or_float")]
    pub price: f64,
    /// Quantity.
    #[serde(with = "string_or_float")]
    pub qty: f64,
    /// Symbol.
    pub symbol: String,
    /// Time.
    pub time: u64,
}

/// Interest history record.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InterestHistoryRecord {
    /// Asset.
    pub asset: String,
    /// Interest.
    #[serde(with = "string_or_float")]
    pub interest: f64,
    /// Interest accrue time.
    pub interest_accured_time: u64,
    /// Interest rate.
    #[serde(with = "string_or_float")]
    pub interest_rate: f64,
    /// Principal.
    #[serde(with = "string_or_float")]
    pub principal: f64,
    /// Interest type.
    #[serde(rename = "type")]
    pub interest_type: String,
    /// Isolated symbol (for isolated margin).
    #[serde(default)]
    pub isolated_symbol: Option<String>,
}

/// Interest rate history record.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InterestRateRecord {
    /// Asset.
    pub asset: String,
    /// Daily interest rate.
    #[serde(with = "string_or_float")]
    pub daily_interest_rate: f64,
    /// Timestamp.
    pub timestamp: u64,
    /// VIP level.
    #[serde(default)]
    pub vip_level: Option<u32>,
}

/// Loan record.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoanRecord {
    /// Asset.
    pub asset: String,
    /// Principal.
    #[serde(with = "string_or_float")]
    pub principal: f64,
    /// Timestamp.
    pub timestamp: u64,
    /// Status (PENDING, CONFIRMED, FAILED).
    pub status: String,
    /// Isolated symbol (for isolated margin).
    #[serde(default)]
    pub isolated_symbol: Option<String>,
    /// Transaction ID.
    pub tx_id: u64,
}

/// Repay record.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepayRecord {
    /// Asset.
    pub asset: String,
    /// Amount.
    #[serde(with = "string_or_float")]
    pub amount: f64,
    /// Interest.
    #[serde(with = "string_or_float")]
    pub interest: f64,
    /// Principal.
    #[serde(with = "string_or_float")]
    pub principal: f64,
    /// Timestamp.
    pub timestamp: u64,
    /// Status (PENDING, CONFIRMED, FAILED).
    pub status: String,
    /// Isolated symbol (for isolated margin).
    #[serde(default)]
    pub isolated_symbol: Option<String>,
    /// Transaction ID.
    pub tx_id: u64,
}

/// Records query result (paginated).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordsQueryResult<T> {
    /// Total records.
    pub total: u64,
    /// Records.
    pub rows: Vec<T>,
}

/// Cross margin pair details.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarginPairDetails {
    /// Symbol ID.
    pub id: u64,
    /// Symbol.
    pub symbol: String,
    /// Base asset.
    pub base: String,
    /// Quote asset.
    pub quote: String,
    /// Whether margin trading is enabled.
    pub is_margin_trade: bool,
    /// Whether buy is allowed.
    pub is_buy_allowed: bool,
    /// Whether sell is allowed.
    pub is_sell_allowed: bool,
}

/// Margin asset info.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarginAssetInfo {
    /// Asset name.
    pub asset_name: String,
    /// Asset full name.
    pub asset_full_name: String,
    /// Whether borrowing is enabled.
    pub is_borrowable: bool,
    /// Whether mortgage is enabled.
    pub is_mortgageable: bool,
    /// User min borrow.
    #[serde(with = "string_or_float")]
    pub user_min_borrow: f64,
    /// User min repay.
    #[serde(with = "string_or_float")]
    pub user_min_repay: f64,
}

/// Margin price index.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarginPriceIndex {
    /// Calculation time.
    pub calc_time: u64,
    /// Price.
    #[serde(with = "string_or_float")]
    pub price: f64,
    /// Symbol.
    pub symbol: String,
}

/// Isolated margin account limit.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IsolatedAccountLimit {
    /// Enabled account count.
    pub enabled_account: u32,
    /// Max account count.
    pub max_account: u32,
}

/// BNB burn status.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BnbBurnStatus {
    /// Whether spot BNB burn is enabled.
    pub spot_bnb_burn: bool,
    /// Whether interest BNB burn is enabled.
    pub interest_bnb_burn: bool,
}

/// Helper for optional f64 fields that may be strings.
mod string_or_float_option {
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(value: &Option<f64>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match value {
            Some(v) => serializer.serialize_str(&v.to_string()),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<f64>, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum StringOrFloat {
            String(String),
            Float(f64),
        }

        let opt: Option<StringOrFloat> = Option::deserialize(deserializer)?;
        match opt {
            Some(StringOrFloat::Float(f)) => Ok(Some(f)),
            Some(StringOrFloat::String(s)) => {
                s.parse::<f64>().map(Some).map_err(serde::de::Error::custom)
            }
            None => Ok(None),
        }
    }
}
