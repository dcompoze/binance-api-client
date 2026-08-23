//! API endpoint implementations.
//!
//! This module contains the implementations for all Binance API endpoints,
//! organized by category.

pub mod account;
pub mod margin;
pub mod market;
#[cfg(feature = "binance-us")]
pub mod userstream;
pub mod wallet;

pub use account::{
    Account, CancelReplaceOrder, CancelReplaceOrderBuilder, NewOcoOrderList, NewOpoOrder,
    NewOpocoOrder, NewOrder, NewOtoOrder, NewOtocoOrder, OcoOrderListBuilder, OpoOrderBuilder,
    OpocoOrderBuilder, OrderBuilder, OtoOrderBuilder, OtocoOrderBuilder,
};
pub use margin::Margin;
pub use market::Market;
#[cfg(feature = "binance-us")]
pub use userstream::UserStream;
pub use wallet::Wallet;
