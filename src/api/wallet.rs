//! Wallet API endpoints (SAPI).
//!
//! This module provides access to Binance Wallet SAPI endpoints for:
//! - System status
//! - Coin information
//! - Deposit/withdrawal operations
//! - Asset management
//! - Universal transfers

use crate::client::Client;
use crate::error::Result;
use crate::models::wallet::{
    AccountSnapshot, AccountSnapshotType, AccountStatus, ApiKeyPermissions, ApiTradingStatus,
    AssetDetail, CoinInfo, DepositAddress, DepositRecord, FundingAsset, SystemStatus, TradeFee,
    TransferHistory, TransferResponse, UniversalTransferType, WalletBalance, WithdrawRecord,
    WithdrawResponse,
};

// SAPI endpoints.
const SAPI_V1_SYSTEM_STATUS: &str = "/sapi/v1/system/status";
const SAPI_V1_CAPITAL_CONFIG_GETALL: &str = "/sapi/v1/capital/config/getall";
const SAPI_V1_ACCOUNT_SNAPSHOT: &str = "/sapi/v1/accountSnapshot";
const SAPI_V1_CAPITAL_DEPOSIT_HISREC: &str = "/sapi/v1/capital/deposit/hisrec";
const SAPI_V1_CAPITAL_DEPOSIT_ADDRESS: &str = "/sapi/v1/capital/deposit/address";
const SAPI_V1_CAPITAL_WITHDRAW_APPLY: &str = "/sapi/v1/capital/withdraw/apply";
const SAPI_V1_CAPITAL_WITHDRAW_HISTORY: &str = "/sapi/v1/capital/withdraw/history";
const SAPI_V1_ASSET_ASSET_DETAIL: &str = "/sapi/v1/asset/assetDetail";
const SAPI_V1_ASSET_TRADE_FEE: &str = "/sapi/v1/asset/tradeFee";
const SAPI_V1_ASSET_TRANSFER: &str = "/sapi/v1/asset/transfer";
const SAPI_V1_ASSET_GET_FUNDING_ASSET: &str = "/sapi/v1/asset/get-funding-asset";
const SAPI_V1_ASSET_WALLET_BALANCE: &str = "/sapi/v1/asset/wallet/balance";
const SAPI_V1_ACCOUNT_STATUS: &str = "/sapi/v1/account/status";
const SAPI_V1_ACCOUNT_API_TRADING_STATUS: &str = "/sapi/v1/account/apiTradingStatus";
const SAPI_V1_ACCOUNT_API_RESTRICTIONS: &str = "/sapi/v1/account/apiRestrictions";

/// Wallet API client.
///
/// Provides access to Binance Wallet SAPI endpoints for asset management,
/// deposits, withdrawals, and account status.
///
/// # Example
///
/// ```rust,ignore
/// let client = Binance::new("api_key", "secret_key")?;
///
/// // Check system status
/// let status = client.wallet().system_status().await?;
/// if status.is_normal() {
///     println!("System is operational");
/// }
///
/// // Get all coin information
/// let coins = client.wallet().all_coins().await?;
/// for coin in coins {
///     println!("{}: free={}", coin.coin, coin.free);
/// }
/// ```
#[derive(Clone)]
pub struct Wallet {
    pub(crate) client: Client,
}

impl Wallet {
    /// Create a new Wallet API client.
    pub(crate) fn new(client: Client) -> Self {
        Self { client }
    }

    // System Status.

    /// Fetch system status.
    ///
    /// Returns whether the Binance system is operational or under maintenance.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let status = client.wallet().system_status().await?;
    /// if status.is_normal() {
    ///     println!("System is operational");
    /// } else {
    ///     println!("System maintenance: {}", status.msg);
    /// }
    /// ```
    pub async fn system_status(&self) -> Result<SystemStatus> {
        self.client.get(SAPI_V1_SYSTEM_STATUS, None).await
    }

    // Coin Information.

    /// Get information of all coins (available for deposit and withdraw).
    ///
    /// Returns detailed information about all supported coins including
    /// deposit/withdraw status, fees, and network information.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let coins = client.wallet().all_coins().await?;
    /// for coin in coins {
    ///     println!("{} ({}): deposit={}, withdraw={}",
    ///         coin.coin, coin.name,
    ///         coin.deposit_all_enable, coin.withdraw_all_enable);
    /// }
    /// ```
    pub async fn all_coins(&self) -> Result<Vec<CoinInfo>> {
        self.client
            .get_signed(SAPI_V1_CAPITAL_CONFIG_GETALL, &[])
            .await
    }

    // Account Snapshots.

    /// Get daily account snapshot.
    ///
    /// Returns account balance snapshots for the specified time period.
    /// The query time period must be less than 30 days.
    /// Only supports querying within the last month.
    ///
    /// # Arguments
    ///
    /// * `snapshot_type` - Type of account (Spot, Margin, or Futures)
    /// * `start_time` - Start timestamp (optional)
    /// * `end_time` - End timestamp (optional)
    /// * `limit` - Number of records (default 7, max 30)
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use binance_api_client::AccountSnapshotType;
    ///
    /// let snapshot = client.wallet()
    ///     .account_snapshot(AccountSnapshotType::Spot, None, None, Some(5))
    ///     .await?;
    /// ```
    pub async fn account_snapshot(
        &self,
        snapshot_type: AccountSnapshotType,
        start_time: Option<u64>,
        end_time: Option<u64>,
        limit: Option<u32>,
    ) -> Result<AccountSnapshot> {
        let type_str = match snapshot_type {
            AccountSnapshotType::Spot => "SPOT",
            AccountSnapshotType::Margin => "MARGIN",
            AccountSnapshotType::Futures => "FUTURES",
        };

        let mut params: Vec<(&str, String)> = vec![("type", type_str.to_string())];

        if let Some(st) = start_time {
            params.push(("startTime", st.to_string()));
        }
        if let Some(et) = end_time {
            params.push(("endTime", et.to_string()));
        }
        if let Some(l) = limit {
            params.push(("limit", l.to_string()));
        }

        let params_ref: Vec<(&str, &str)> = params.iter().map(|(k, v)| (*k, v.as_str())).collect();
        self.client
            .get_signed(SAPI_V1_ACCOUNT_SNAPSHOT, &params_ref)
            .await
    }

    // Deposit.

    /// Get deposit address for a coin.
    ///
    /// # Arguments
    ///
    /// * `coin` - Coin symbol (e.g., "BTC")
    /// * `network` - Network to use (optional, uses default if not specified)
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let address = client.wallet().deposit_address("BTC", None).await?;
    /// println!("Deposit to: {}", address.address);
    /// ```
    pub async fn deposit_address(
        &self,
        coin: &str,
        network: Option<&str>,
    ) -> Result<DepositAddress> {
        let mut params: Vec<(&str, String)> = vec![("coin", coin.to_string())];

        if let Some(n) = network {
            params.push(("network", n.to_string()));
        }

        let params_ref: Vec<(&str, &str)> = params.iter().map(|(k, v)| (*k, v.as_str())).collect();
        self.client
            .get_signed(SAPI_V1_CAPITAL_DEPOSIT_ADDRESS, &params_ref)
            .await
    }

    /// Get deposit history.
    ///
    /// # Arguments
    ///
    /// * `coin` - Filter by coin (optional)
    /// * `status` - Filter by status: 0=pending, 6=credited, 1=success (optional)
    /// * `start_time` - Start timestamp (optional)
    /// * `end_time` - End timestamp (optional)
    /// * `offset` - Pagination offset (optional)
    /// * `limit` - Number of records (default 1000, max 1000)
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let deposits = client.wallet()
    ///     .deposit_history(Some("BTC"), None, None, None, None, Some(10))
    ///     .await?;
    /// for deposit in deposits {
    ///     println!("{}: {} {}", deposit.tx_id, deposit.amount, deposit.coin);
    /// }
    /// ```
    pub async fn deposit_history(
        &self,
        coin: Option<&str>,
        status: Option<u32>,
        start_time: Option<u64>,
        end_time: Option<u64>,
        offset: Option<u32>,
        limit: Option<u32>,
    ) -> Result<Vec<DepositRecord>> {
        let mut params: Vec<(&str, String)> = vec![];

        if let Some(c) = coin {
            params.push(("coin", c.to_string()));
        }
        if let Some(s) = status {
            params.push(("status", s.to_string()));
        }
        if let Some(st) = start_time {
            params.push(("startTime", st.to_string()));
        }
        if let Some(et) = end_time {
            params.push(("endTime", et.to_string()));
        }
        if let Some(o) = offset {
            params.push(("offset", o.to_string()));
        }
        if let Some(l) = limit {
            params.push(("limit", l.to_string()));
        }

        let params_ref: Vec<(&str, &str)> = params.iter().map(|(k, v)| (*k, v.as_str())).collect();
        self.client
            .get_signed(SAPI_V1_CAPITAL_DEPOSIT_HISREC, &params_ref)
            .await
    }

    // Withdrawal.

    /// Submit a withdrawal request.
    ///
    /// # Arguments
    ///
    /// * `coin` - Coin symbol
    /// * `address` - Withdrawal address
    /// * `amount` - Amount to withdraw
    /// * `network` - Network to use (optional)
    /// * `address_tag` - Secondary address identifier (memo/tag, optional)
    /// * `withdraw_order_id` - Client ID for the withdrawal (optional)
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let response = client.wallet()
    ///     .withdraw("USDT", "0x1234...", "100.0", Some("ETH"), None, None)
    ///     .await?;
    /// println!("Withdrawal ID: {}", response.id);
    /// ```
    pub async fn withdraw(
        &self,
        coin: &str,
        address: &str,
        amount: &str,
        network: Option<&str>,
        address_tag: Option<&str>,
        withdraw_order_id: Option<&str>,
    ) -> Result<WithdrawResponse> {
        let mut params: Vec<(&str, String)> = vec![
            ("coin", coin.to_string()),
            ("address", address.to_string()),
            ("amount", amount.to_string()),
        ];

        if let Some(n) = network {
            params.push(("network", n.to_string()));
        }
        if let Some(tag) = address_tag {
            params.push(("addressTag", tag.to_string()));
        }
        if let Some(id) = withdraw_order_id {
            params.push(("withdrawOrderId", id.to_string()));
        }

        let params_ref: Vec<(&str, &str)> = params.iter().map(|(k, v)| (*k, v.as_str())).collect();
        self.client
            .post_signed(SAPI_V1_CAPITAL_WITHDRAW_APPLY, &params_ref)
            .await
    }

    /// Get withdrawal history.
    ///
    /// # Arguments
    ///
    /// * `coin` - Filter by coin (optional)
    /// * `withdraw_order_id` - Filter by client withdrawal ID (optional)
    /// * `status` - Filter by status (optional)
    /// * `start_time` - Start timestamp (optional)
    /// * `end_time` - End timestamp (optional)
    /// * `offset` - Pagination offset (optional)
    /// * `limit` - Number of records (default 1000, max 1000)
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let withdrawals = client.wallet()
    ///     .withdraw_history(None, None, None, None, None, None, Some(10))
    ///     .await?;
    /// ```
    #[allow(clippy::too_many_arguments)]
    pub async fn withdraw_history(
        &self,
        coin: Option<&str>,
        withdraw_order_id: Option<&str>,
        status: Option<u32>,
        start_time: Option<u64>,
        end_time: Option<u64>,
        offset: Option<u32>,
        limit: Option<u32>,
    ) -> Result<Vec<WithdrawRecord>> {
        let mut params: Vec<(&str, String)> = vec![];

        if let Some(c) = coin {
            params.push(("coin", c.to_string()));
        }
        if let Some(id) = withdraw_order_id {
            params.push(("withdrawOrderId", id.to_string()));
        }
        if let Some(s) = status {
            params.push(("status", s.to_string()));
        }
        if let Some(st) = start_time {
            params.push(("startTime", st.to_string()));
        }
        if let Some(et) = end_time {
            params.push(("endTime", et.to_string()));
        }
        if let Some(o) = offset {
            params.push(("offset", o.to_string()));
        }
        if let Some(l) = limit {
            params.push(("limit", l.to_string()));
        }

        let params_ref: Vec<(&str, &str)> = params.iter().map(|(k, v)| (*k, v.as_str())).collect();
        self.client
            .get_signed(SAPI_V1_CAPITAL_WITHDRAW_HISTORY, &params_ref)
            .await
    }

    // Asset Management.

    /// Get asset detail (deposit/withdraw fees and status).
    ///
    /// # Arguments
    ///
    /// * `asset` - Asset symbol (optional, returns all if not specified)
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let details = client.wallet().asset_detail(Some("BTC")).await?;
    /// ```
    pub async fn asset_detail(
        &self,
        asset: Option<&str>,
    ) -> Result<std::collections::HashMap<String, AssetDetail>> {
        let mut params: Vec<(&str, String)> = vec![];

        if let Some(a) = asset {
            params.push(("asset", a.to_string()));
        }

        let params_ref: Vec<(&str, &str)> = params.iter().map(|(k, v)| (*k, v.as_str())).collect();
        self.client
            .get_signed(SAPI_V1_ASSET_ASSET_DETAIL, &params_ref)
            .await
    }

    /// Get trade fee for symbols.
    ///
    /// # Arguments
    ///
    /// * `symbol` - Trading pair symbol (optional, returns all if not specified)
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let fees = client.wallet().trade_fee(Some("BTCUSDT")).await?;
    /// for fee in fees {
    ///     println!("{}: maker={}, taker={}",
    ///         fee.symbol, fee.maker_commission, fee.taker_commission);
    /// }
    /// ```
    pub async fn trade_fee(&self, symbol: Option<&str>) -> Result<Vec<TradeFee>> {
        let mut params: Vec<(&str, String)> = vec![];

        if let Some(s) = symbol {
            params.push(("symbol", s.to_string()));
        }

        let params_ref: Vec<(&str, &str)> = params.iter().map(|(k, v)| (*k, v.as_str())).collect();
        self.client
            .get_signed(SAPI_V1_ASSET_TRADE_FEE, &params_ref)
            .await
    }

    // Universal Transfer.

    /// Execute a universal transfer between accounts.
    ///
    /// # Arguments
    ///
    /// * `transfer_type` - Type of transfer
    /// * `asset` - Asset to transfer
    /// * `amount` - Amount to transfer
    /// * `from_symbol` - Required for isolated margin transfers
    /// * `to_symbol` - Required for isolated margin transfers
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use binance_api_client::UniversalTransferType;
    ///
    /// let response = client.wallet()
    ///     .universal_transfer(
    ///         UniversalTransferType::MainFunding,
    ///         "USDT",
    ///         "100.0",
    ///         None,
    ///         None,
    ///     )
    ///     .await?;
    /// println!("Transfer ID: {}", response.tran_id);
    /// ```
    pub async fn universal_transfer(
        &self,
        transfer_type: UniversalTransferType,
        asset: &str,
        amount: &str,
        from_symbol: Option<&str>,
        to_symbol: Option<&str>,
    ) -> Result<TransferResponse> {
        let type_str = transfer_type.as_str().to_string();

        let mut params: Vec<(&str, String)> = vec![
            ("type", type_str),
            ("asset", asset.to_string()),
            ("amount", amount.to_string()),
        ];

        if let Some(from) = from_symbol {
            params.push(("fromSymbol", from.to_string()));
        }
        if let Some(to) = to_symbol {
            params.push(("toSymbol", to.to_string()));
        }

        let params_ref: Vec<(&str, &str)> = params.iter().map(|(k, v)| (*k, v.as_str())).collect();
        self.client
            .post_signed(SAPI_V1_ASSET_TRANSFER, &params_ref)
            .await
    }

    /// Get universal transfer history.
    ///
    /// # Arguments
    ///
    /// * `transfer_type` - Type of transfer to query
    /// * `start_time` - Start timestamp (optional)
    /// * `end_time` - End timestamp (optional)
    /// * `current` - Page number (default 1)
    /// * `size` - Page size (default 10, max 100)
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use binance_api_client::UniversalTransferType;
    ///
    /// let history = client.wallet()
    ///     .transfer_history(UniversalTransferType::MainFunding, None, None, None, Some(10))
    ///     .await?;
    /// ```
    pub async fn transfer_history(
        &self,
        transfer_type: UniversalTransferType,
        start_time: Option<u64>,
        end_time: Option<u64>,
        current: Option<u32>,
        size: Option<u32>,
    ) -> Result<TransferHistory> {
        let type_str = transfer_type.as_str().to_string();

        let mut params: Vec<(&str, String)> = vec![("type", type_str)];

        if let Some(st) = start_time {
            params.push(("startTime", st.to_string()));
        }
        if let Some(et) = end_time {
            params.push(("endTime", et.to_string()));
        }
        if let Some(c) = current {
            params.push(("current", c.to_string()));
        }
        if let Some(s) = size {
            params.push(("size", s.to_string()));
        }

        let params_ref: Vec<(&str, &str)> = params.iter().map(|(k, v)| (*k, v.as_str())).collect();
        self.client
            .get_signed(SAPI_V1_ASSET_TRANSFER, &params_ref)
            .await
    }

    // Wallet Balances.

    /// Get funding wallet balance.
    ///
    /// # Arguments
    ///
    /// * `asset` - Asset to query (optional, returns all if not specified)
    /// * `need_btc_valuation` - Whether to include BTC valuation (optional)
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let assets = client.wallet().funding_wallet(None, Some(true)).await?;
    /// for asset in assets {
    ///     println!("{}: {}", asset.asset, asset.free);
    /// }
    /// ```
    pub async fn funding_wallet(
        &self,
        asset: Option<&str>,
        need_btc_valuation: Option<bool>,
    ) -> Result<Vec<FundingAsset>> {
        let mut params: Vec<(&str, String)> = vec![];

        if let Some(a) = asset {
            params.push(("asset", a.to_string()));
        }
        if let Some(btc) = need_btc_valuation {
            params.push(("needBtcValuation", btc.to_string()));
        }

        let params_ref: Vec<(&str, &str)> = params.iter().map(|(k, v)| (*k, v.as_str())).collect();
        self.client
            .post_signed(SAPI_V1_ASSET_GET_FUNDING_ASSET, &params_ref)
            .await
    }

    /// Get wallet balance for all asset wallets.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let balances = client.wallet().wallet_balance().await?;
    /// for balance in balances {
    ///     if balance.balance > 0.0 {
    ///         println!("{}: {}", balance.wallet_name, balance.balance);
    ///     }
    /// }
    /// ```
    pub async fn wallet_balance(&self) -> Result<Vec<WalletBalance>> {
        self.client
            .get_signed(SAPI_V1_ASSET_WALLET_BALANCE, &[])
            .await
    }

    // Account Status.

    /// Get account status.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let status = client.wallet().account_status().await?;
    /// println!("Account status: {}", status.data);
    /// ```
    pub async fn account_status(&self) -> Result<AccountStatus> {
        self.client.get_signed(SAPI_V1_ACCOUNT_STATUS, &[]).await
    }

    /// Get API trading status.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let status = client.wallet().api_trading_status().await?;
    /// if status.data.is_locked {
    ///     println!("Trading is locked!");
    /// }
    /// ```
    pub async fn api_trading_status(&self) -> Result<ApiTradingStatus> {
        self.client
            .get_signed(SAPI_V1_ACCOUNT_API_TRADING_STATUS, &[])
            .await
    }

    /// Get API key permissions.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let permissions = client.wallet().api_key_permissions().await?;
    /// println!("Can trade: {}", permissions.enable_spot_and_margin_trading);
    /// println!("Can withdraw: {}", permissions.enable_withdrawals);
    /// ```
    pub async fn api_key_permissions(&self) -> Result<ApiKeyPermissions> {
        self.client
            .get_signed(SAPI_V1_ACCOUNT_API_RESTRICTIONS, &[])
            .await
    }
}
