//! Wallet API response models.
//!
//! Models for the Binance Wallet SAPI endpoints.

use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

use super::string_or_float;

/// System status response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStatus {
    /// Status: 0 = normal, 1 = system maintenance
    pub status: u32,
    /// Status message (e.g., "normal", "system_maintenance")
    pub msg: String,
}

impl SystemStatus {
    /// Returns true if the system is operating normally.
    pub fn is_normal(&self) -> bool {
        self.status == 0
    }
}

/// Coin network information.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoinNetwork {
    /// Address regex pattern.
    #[serde(default)]
    pub address_regex: Option<String>,
    /// Coin name.
    pub coin: String,
    /// Deposit description.
    #[serde(default)]
    pub deposit_desc: Option<String>,
    /// Whether deposits are enabled.
    pub deposit_enable: bool,
    /// Whether this is the default network.
    pub is_default: bool,
    /// Memo regex pattern.
    #[serde(default)]
    pub memo_regex: Option<String>,
    /// Minimum confirmations for deposit.
    pub min_confirm: u32,
    /// Network name.
    pub name: String,
    /// Network identifier.
    pub network: String,
    /// Whether special tips are available.
    #[serde(default)]
    pub special_tips: Option<String>,
    /// Unlock confirmations required.
    #[serde(default)]
    pub un_lock_confirm: Option<u32>,
    /// Withdraw description.
    #[serde(default)]
    pub withdraw_desc: Option<String>,
    /// Whether withdrawals are enabled.
    pub withdraw_enable: bool,
    /// Withdrawal fee.
    #[serde(with = "string_or_float")]
    pub withdraw_fee: f64,
    /// Withdrawal integer multiple.
    #[serde(default, with = "string_or_float_option")]
    pub withdraw_integer_multiple: Option<f64>,
    /// Maximum withdrawal amount.
    #[serde(with = "string_or_float")]
    pub withdraw_max: f64,
    /// Minimum withdrawal amount.
    #[serde(with = "string_or_float")]
    pub withdraw_min: f64,
    /// Whether same address is supported.
    #[serde(default)]
    pub same_address: Option<bool>,
    /// Estimated arrival time.
    #[serde(default)]
    pub estimated_arrival_time: Option<u64>,
    /// Whether the network is busy.
    #[serde(default)]
    pub busy: Option<bool>,
}

/// Coin information from wallet config.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoinInfo {
    /// Coin symbol (e.g., "BTC").
    pub coin: String,
    /// Whether deposit is available for all networks.
    pub deposit_all_enable: bool,
    /// Free balance.
    #[serde(with = "string_or_float")]
    pub free: f64,
    /// Freeze balance.
    #[serde(with = "string_or_float")]
    pub freeze: f64,
    /// IPO-able balance.
    #[serde(with = "string_or_float")]
    pub ipoable: f64,
    /// IPOING balance.
    #[serde(with = "string_or_float")]
    pub ipoing: f64,
    /// Whether legal money.
    pub is_legal_money: bool,
    /// Locked balance.
    #[serde(with = "string_or_float")]
    pub locked: f64,
    /// Full coin name.
    pub name: String,
    /// Available networks for this coin.
    pub network_list: Vec<CoinNetwork>,
    /// Storage balance.
    #[serde(with = "string_or_float")]
    pub storage: f64,
    /// Whether trading is enabled.
    pub trading: bool,
    /// Whether withdraw is available for all networks.
    pub withdraw_all_enable: bool,
    /// Withdrawing balance.
    #[serde(with = "string_or_float")]
    pub withdrawing: f64,
}

/// Deposit address information.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DepositAddress {
    /// Deposit address.
    pub address: String,
    /// Coin symbol.
    pub coin: String,
    /// Tag/memo (if applicable).
    pub tag: String,
    /// URL for address (optional).
    #[serde(default)]
    pub url: Option<String>,
}

/// Deposit record from history.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DepositRecord {
    /// Deposit amount.
    #[serde(with = "string_or_float")]
    pub amount: f64,
    /// Coin symbol.
    pub coin: String,
    /// Network used.
    pub network: String,
    /// Deposit status.
    pub status: DepositStatus,
    /// Deposit address.
    pub address: String,
    /// Address tag (if applicable).
    #[serde(default)]
    pub address_tag: Option<String>,
    /// Transaction ID.
    pub tx_id: String,
    /// Insert time (timestamp).
    pub insert_time: u64,
    /// Transfer type.
    #[serde(default)]
    pub transfer_type: Option<u32>,
    /// Confirm times.
    #[serde(default)]
    pub confirm_times: Option<String>,
    /// Unlock confirm.
    #[serde(default)]
    pub unlock_confirm: Option<u32>,
    /// Unique ID.
    #[serde(default)]
    pub id: Option<String>,
}

/// Deposit status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum DepositStatus {
    /// Pending
    Pending = 0,
    /// Success
    Success = 1,
    /// Success (credited but cannot withdraw)
    CreditedCannotWithdraw = 6,
}

/// Withdrawal record from history.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WithdrawRecord {
    /// Withdrawal address.
    pub address: String,
    /// Amount.
    #[serde(with = "string_or_float")]
    pub amount: f64,
    /// Apply time.
    pub apply_time: String,
    /// Coin symbol.
    pub coin: String,
    /// Withdrawal ID.
    pub id: String,
    /// Withdraw order ID (user-supplied).
    #[serde(default)]
    pub withdraw_order_id: Option<String>,
    /// Network used.
    pub network: String,
    /// Transfer type.
    #[serde(default)]
    pub transfer_type: Option<u32>,
    /// Status.
    pub status: WithdrawStatus,
    /// Transaction fee.
    #[serde(with = "string_or_float")]
    pub transaction_fee: f64,
    /// Confirm number.
    #[serde(default)]
    pub confirm_no: Option<u32>,
    /// Additional info.
    #[serde(default)]
    pub info: Option<String>,
    /// Transaction ID.
    #[serde(default)]
    pub tx_id: Option<String>,
}

/// Withdrawal status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum WithdrawStatus {
    /// Email sent
    EmailSent = 0,
    /// Cancelled
    Cancelled = 1,
    /// Awaiting approval
    AwaitingApproval = 2,
    /// Rejected
    Rejected = 3,
    /// Processing
    Processing = 4,
    /// Failure
    Failure = 5,
    /// Completed
    Completed = 6,
}

/// Withdrawal request response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WithdrawResponse {
    /// Withdrawal ID.
    pub id: String,
}

/// Asset detail information.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetDetail {
    /// Minimum withdrawal amount.
    #[serde(with = "string_or_float")]
    pub min_withdraw_amount: f64,
    /// Whether deposit is enabled.
    pub deposit_status: bool,
    /// Withdrawal fee.
    #[serde(with = "string_or_float")]
    pub withdraw_fee: f64,
    /// Whether withdrawal is enabled.
    pub withdraw_status: bool,
    /// Deposit tip (optional).
    #[serde(default)]
    pub deposit_tip: Option<String>,
}

/// Trade fee information.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TradeFee {
    /// Symbol.
    pub symbol: String,
    /// Maker commission.
    #[serde(with = "string_or_float")]
    pub maker_commission: f64,
    /// Taker commission.
    #[serde(with = "string_or_float")]
    pub taker_commission: f64,
}

/// Universal transfer type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum UniversalTransferType {
    /// Spot to USDM Futures
    MainUmfuture,
    /// Spot to COINM Futures
    MainCmfuture,
    /// Spot to Margin (cross)
    MainMargin,
    /// USDM Futures to Spot
    UmfutureMain,
    /// USDM Futures to Margin (cross)
    UmfutureMargin,
    /// COINM Futures to Spot
    CmfutureMain,
    /// COINM Futures to Margin (cross)
    CmfutureMargin,
    /// Margin (cross) to Spot
    MarginMain,
    /// Margin (cross) to USDM Futures
    MarginUmfuture,
    /// Margin (cross) to COINM Futures
    MarginCmfuture,
    /// Spot to Isolated Margin
    MainIsolatedMargin,
    /// Isolated Margin to Spot
    IsolatedMarginMain,
    /// Isolated Margin to Isolated Margin
    IsolatedMarginIsolatedMargin,
    /// Spot to Funding
    MainFunding,
    /// Funding to Spot
    FundingMain,
    /// Funding to USDM Futures
    FundingUmfuture,
    /// USDM Futures to Funding
    UmfutureFunding,
    /// Margin (cross) to Funding
    MarginFunding,
    /// Funding to Margin (cross)
    FundingMargin,
    /// Funding to COINM Futures
    FundingCmfuture,
    /// COINM Futures to Funding
    CmfutureFunding,
}

impl UniversalTransferType {
    /// Return the API wire value for this transfer type.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::MainUmfuture => "MAIN_UMFUTURE",
            Self::MainCmfuture => "MAIN_CMFUTURE",
            Self::MainMargin => "MAIN_MARGIN",
            Self::UmfutureMain => "UMFUTURE_MAIN",
            Self::UmfutureMargin => "UMFUTURE_MARGIN",
            Self::CmfutureMain => "CMFUTURE_MAIN",
            Self::CmfutureMargin => "CMFUTURE_MARGIN",
            Self::MarginMain => "MARGIN_MAIN",
            Self::MarginUmfuture => "MARGIN_UMFUTURE",
            Self::MarginCmfuture => "MARGIN_CMFUTURE",
            Self::MainIsolatedMargin => "MAIN_ISOLATED_MARGIN",
            Self::IsolatedMarginMain => "ISOLATED_MARGIN_MAIN",
            Self::IsolatedMarginIsolatedMargin => "ISOLATED_MARGIN_ISOLATED_MARGIN",
            Self::MainFunding => "MAIN_FUNDING",
            Self::FundingMain => "FUNDING_MAIN",
            Self::FundingUmfuture => "FUNDING_UMFUTURE",
            Self::UmfutureFunding => "UMFUTURE_FUNDING",
            Self::MarginFunding => "MARGIN_FUNDING",
            Self::FundingMargin => "FUNDING_MARGIN",
            Self::FundingCmfuture => "FUNDING_CMFUTURE",
            Self::CmfutureFunding => "CMFUTURE_FUNDING",
        }
    }
}

/// Universal transfer response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferResponse {
    /// Transaction ID.
    pub tran_id: u64,
}

/// Universal transfer record.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferRecord {
    /// Asset.
    pub asset: String,
    /// Amount.
    #[serde(with = "string_or_float")]
    pub amount: f64,
    /// Transfer type.
    #[serde(rename = "type")]
    pub transfer_type: UniversalTransferType,
    /// Status.
    pub status: String,
    /// Transaction ID.
    pub tran_id: u64,
    /// Timestamp.
    pub timestamp: u64,
}

/// Transfer history response (paginated).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferHistory {
    /// Total count.
    pub total: u64,
    /// Transfer records.
    pub rows: Vec<TransferRecord>,
}

/// Wallet balance entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WalletBalance {
    /// Whether balance is active.
    pub activate: bool,
    /// Balance amount.
    #[serde(with = "string_or_float")]
    pub balance: f64,
    /// Wallet name.
    pub wallet_name: String,
}

/// Funding wallet asset.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FundingAsset {
    /// Asset.
    pub asset: String,
    /// Free balance.
    #[serde(with = "string_or_float")]
    pub free: f64,
    /// Locked balance.
    #[serde(with = "string_or_float")]
    pub locked: f64,
    /// Freeze balance.
    #[serde(with = "string_or_float")]
    pub freeze: f64,
    /// Withdrawing balance.
    #[serde(with = "string_or_float")]
    pub withdrawing: f64,
    /// BTC valuation (optional).
    #[serde(default, with = "string_or_float_option")]
    pub btc_valuation: Option<f64>,
}

/// Account snapshot type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AccountSnapshotType {
    /// Spot account
    Spot,
    /// Margin account
    Margin,
    /// Futures account
    Futures,
}

/// Account snapshot response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountSnapshot {
    /// Response code.
    pub code: i32,
    /// Response message.
    pub msg: String,
    /// Snapshot data.
    pub snapshot_vos: Vec<SnapshotData>,
}

/// Snapshot data entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SnapshotData {
    /// Snapshot type.
    #[serde(rename = "type")]
    pub snapshot_type: String,
    /// Update time.
    pub update_time: u64,
    /// Snapshot data (varies by type).
    pub data: serde_json::Value,
}

/// API key permissions.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiKeyPermissions {
    /// Whether IP restricted.
    pub ip_restrict: bool,
    /// Creation time.
    pub create_time: u64,
    /// Whether spot trading is enabled.
    pub enable_spot_and_margin_trading: bool,
    /// Whether withdrawals are enabled.
    pub enable_withdrawals: bool,
    /// Whether internal transfers are enabled.
    pub enable_internal_transfer: bool,
    /// Permits universal transfer.
    pub permits_universal_transfer: bool,
    /// Whether vanilla options are enabled.
    pub enable_vanilla_options: bool,
    /// Whether reading is enabled.
    pub enable_reading: bool,
    /// Whether futures trading is enabled.
    pub enable_futures: bool,
    /// Whether margin loan/borrow/repay is enabled.
    pub enable_margin: bool,
    /// Trading authority expiration time.
    #[serde(default)]
    pub trading_authority_expiration_time: Option<u64>,
}

/// Account status.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountStatus {
    /// Account status data.
    pub data: String,
}

/// API trading status.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiTradingStatus {
    /// Status data.
    pub data: ApiTradingStatusData,
}

/// API trading status data.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiTradingStatusData {
    /// Is locked.
    pub is_locked: bool,
    /// Planned recovery time (if locked).
    #[serde(default)]
    pub planned_recover_time: Option<u64>,
    /// Trigger condition.
    pub trigger_condition: serde_json::Value,
    /// Update time.
    pub update_time: u64,
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
