//! Core data types for the transaction contract.
//!
//! This module is a pure data-definition module: it declares the public
//! [`Transaction`] type returned by queries, the packed
//! [`StoredTransaction`] form persisted to ledger storage, and the
//! [`CallbackPayload`] argument accepted by `register_callback` /
//! `batch_register_callback`. The pack/unpack conversion logic lives in
//! `storage.rs`.

use soroban_sdk::{contracttype, Address, Bytes, String, Vec};

/// Lifecycle status of a transaction.
///
/// Mirrors the "Transaction lifecycle (on-chain)" diagram in `README.md`:
/// `Pending -> Processing -> Completed`, with `Failed` and `Cancelled` as two
/// independent terminal branches. Discriminant values are stable across
/// upgrades and must not be reordered, since they are packed into the low
/// 4 bits of [`StoredTransaction::packed`].
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum TransactionStatus {
    /// Transaction has been registered but not yet processed.
    Pending = 0,
    /// Transaction is actively being processed.
    Processing = 1,
    /// Transaction completed successfully (terminal).
    Completed = 2,
    /// Transaction failed permanently (terminal).
    Failed = 3,
    /// Transaction was cancelled (terminal).
    Cancelled = 4,
}

/// Kind of callback registered for a transaction.
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum CallbackType {
    /// No callback registered.
    None = 0,
    /// Callback invoked on successful completion.
    OnSuccess = 1,
    /// Callback invoked on failure.
    OnFailure = 2,
    /// Callback invoked on any terminal state.
    OnComplete = 3,
}

/// Public, ergonomic representation of a transaction returned by
/// `get_transaction`.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Transaction {
    /// Unique transaction identifier.
    pub tx_id: Bytes,
    /// Current lifecycle status.
    pub status: TransactionStatus,
    /// Callback kind registered for this transaction.
    pub callback_type: CallbackType,
    /// Amount transferred.
    pub amount: i128,
    /// Destination Stellar account.
    pub stellar_account: Address,
    /// Asset code (e.g. "USDC").
    pub asset_code: String,
    /// Asset issuer address.
    pub asset_issuer: Address,
    /// Number of retries attempted so far.
    pub retry_count: u32,
    /// Arbitrary tags attached to the transaction.
    pub tags: Vec<String>,
}

/// Packed on-ledger representation of a [`Transaction`].
///
/// `status`, `callback_type`, and `retry_count` are bit-packed into a single
/// `u32` (`packed`) to save ledger bytes. Layout (least-significant first):
///
/// - bits 0..=3   : `status` (4 bits, up to 16 variants)
/// - bits 4..=7   : `callback_type` (4 bits, up to 16 variants)
/// - bits 8..=31  : `retry_count` (24 bits, up to 16,777,215 retries)
///
/// The remaining fields are stored verbatim. Use the conversions in
/// `storage.rs` to move between this and [`Transaction`].
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StoredTransaction {
    /// Unique transaction identifier.
    pub tx_id: Bytes,
    /// Bit-packed `status` | `callback_type` | `retry_count`.
    pub packed: u32,
    /// Amount transferred.
    pub amount: i128,
    /// Destination Stellar account.
    pub stellar_account: Address,
    /// Asset code.
    pub asset_code: String,
    /// Asset issuer address.
    pub asset_issuer: Address,
    /// Arbitrary tags attached to the transaction.
    pub tags: Vec<String>,
}

/// Argument accepted by `register_callback` and `batch_register_callback`.
///
/// This is the hand-off contract between the off-chain relay and this
/// contract. Its field set is exactly the set of fields that
/// `THREAT_MODEL.md` §4.2 documents as validated on `register_callback`:
///
/// - `tx_id` — idempotency / transaction identifier used by dedup (issue #22)
///   to reject duplicate registrations.
/// - `amount` — checked against the configured amount bounds.
/// - `stellar_account` — validated as a well-formed strkey account.
/// - `asset_code` — length-bounded asset code (e.g. "USDC").
/// - `asset_issuer` — validated as a well-formed strkey issuer.
///
/// The struct is intentionally lean: field ordering and count feed into the
/// batch-cost resource budget (see the `5f66065` commit message, "3 of the 25
/// write entries"), so do not pad it speculatively.
///
/// # Mapping to storage
///
/// A validated `CallbackPayload` maps deterministically onto the stored
/// [`Transaction`] / [`StoredTransaction`] (issue #11):
///
/// - `tx_id`            -> `Transaction::tx_id` / `StoredTransaction::tx_id`
/// - `amount`           -> `Transaction::amount` / `StoredTransaction::amount`
/// - `stellar_account`  -> `Transaction::stellar_account` /
///   `StoredTransaction::stellar_account`
/// - `asset_code`       -> `Transaction::asset_code` /
///   `StoredTransaction::asset_code`
/// - `asset_issuer`     -> `Transaction::asset_issuer` /
///   `StoredTransaction::asset_issuer`
///
/// Fields not present on the payload take their defaults: `status` is
/// `Pending`, `callback_type` is `None`, `retry_count` is `0`, and `tags` is
/// empty.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CallbackPayload {
    /// Idempotency / transaction identifier used for dedup.
    pub tx_id: Bytes,
    /// Amount transferred; validated against the configured bounds.
    pub amount: i128,
    /// Destination Stellar account; validated as a strkey account.
    pub stellar_account: Address,
    /// Asset code (e.g. "USDC"); length-bounded.
    pub asset_code: String,
    /// Asset issuer address; validated as a strkey issuer.
    pub asset_issuer: Address,
}
