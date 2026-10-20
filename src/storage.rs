//! Ledger storage helpers for the transaction contract.
//!
//! This module owns the pack/unpack conversion between the public
//! [`Transaction`] type and the packed [`StoredTransaction`] form persisted to
//! ledger storage.

use crate::types::{CallbackType, StoredTransaction, Transaction, TransactionStatus};

/// Bit offset of the `status` field within `StoredTransaction::packed`.
const STATUS_SHIFT: u32 = 0;
/// Bit mask for the `status` field (4 bits).
const STATUS_MASK: u32 = 0xF;

/// Bit offset of the `callback_type` field within `StoredTransaction::packed`.
const CALLBACK_SHIFT: u32 = 4;
/// Bit mask for the `callback_type` field (4 bits).
const CALLBACK_MASK: u32 = 0xF;

/// Bit offset of the `retry_count` field within `StoredTransaction::packed`.
const RETRY_SHIFT: u32 = 8;
/// Bit mask for the `retry_count` field (24 bits).
const RETRY_MASK: u32 = 0xFF_FFFF;

/// Map a `u32` discriminant back to a [`TransactionStatus`].
fn status_from_u32(value: u32) -> TransactionStatus {
    match value {
        1 => TransactionStatus::Completed,
        2 => TransactionStatus::Failed,
        3 => TransactionStatus::Retrying,
        4 => TransactionStatus::Cancelled,
        _ => TransactionStatus::Pending,
    }
}

/// Map a `u32` discriminant back to a [`CallbackType`].
fn callback_from_u32(value: u32) -> CallbackType {
    match value {
        1 => CallbackType::OnSuccess,
        2 => CallbackType::OnFailure,
        3 => CallbackType::OnComplete,
        _ => CallbackType::None,
    }
}

/// Pack a [`Transaction`] into its on-ledger [`StoredTransaction`] form.
pub fn pack(tx: &Transaction) -> StoredTransaction {
    let packed = ((tx.status as u32 & STATUS_MASK) << STATUS_SHIFT)
        | ((tx.callback_type as u32 & CALLBACK_MASK) << CALLBACK_SHIFT)
        | ((tx.retry_count & RETRY_MASK) << RETRY_SHIFT);

    StoredTransaction {
        tx_id: tx.tx_id.clone(),
        packed,
        amount: tx.amount,
        stellar_account: tx.stellar_account.clone(),
        asset_code: tx.asset_code.clone(),
        asset_issuer: tx.asset_issuer.clone(),
        tags: tx.tags.clone(),
    }
}

/// Unpack a [`StoredTransaction`] back into the public [`Transaction`] form.
pub fn unpack(stored: &StoredTransaction) -> Transaction {
    let status = status_from_u32((stored.packed >> STATUS_SHIFT) & STATUS_MASK);
    let callback_type = callback_from_u32((stored.packed >> CALLBACK_SHIFT) & CALLBACK_MASK);
    let retry_count = (stored.packed >> RETRY_SHIFT) & RETRY_MASK;

    Transaction {
        tx_id: stored.tx_id.clone(),
        status,
        callback_type,
        amount: stored.amount,
        stellar_account: stored.stellar_account.clone(),
        asset_code: stored.asset_code.clone(),
        asset_issuer: stored.asset_issuer.clone(),
        retry_count,
        tags: stored.tags.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{Address, Bytes, Env, String, Vec};

    #[test]
    fn round_trip_preserves_every_field() {
        let env = Env::default();
        let tx = Transaction {
            tx_id: Bytes::from_array(&env, &[1, 2, 3, 4]),
            status: TransactionStatus::Retrying,
            callback_type: CallbackType::OnFailure,
            amount: 1_000_000,
            stellar_account: Address::generate(&env),
            asset_code: String::from_str(&env, "USDC"),
            asset_issuer: Address::generate(&env),
            retry_count: 42,
            tags: Vec::new(&env),
        };

        let stored = pack(&tx);
        let restored = unpack(&stored);

        assert_eq!(restored, tx);
    }
}
