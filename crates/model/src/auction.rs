//! Module defining a batch auction.

use crate::{order::Order, u256_decimal::DecimalU256};
use primitive_types::{H160, U256};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use std::collections::BTreeMap;

pub type AuctionId = i64;

// Separate type because we usually use the id but store it in the database without as the id is a
// separate column and is autogenerated when we insert the auction.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuctionWithId {
    /// Increments whenever the backend updates the auction.
    ///
    /// Will eventually be synchronized with solution submission for https://github.com/cowprotocol/services/issues/230 .
    pub id: AuctionId,
    #[serde(flatten)]
    pub auction: Auction,
}

/// A batch auction.
#[serde_as]
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Auction {
    /// The block that this auction is valid for.
    /// The block number for the auction. Orders and prices are guaranteed to be
    /// valid on this block.
    pub block: u64,

    /// The latest block on which a settlement has been processed.
    ///
    /// Note that under certain conditions it is possible for a settlement to
    /// have been mined as part of [`block`] but not have yet been processed.
    pub latest_settlement_block: u64,

    /// The solvable orders included in the auction.
    pub orders: Vec<Order>,

    /// The reference prices for all traded tokens in the auction.
    #[serde_as(as = "BTreeMap<_, DecimalU256>")]
    pub prices: BTreeMap<H160, U256>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::order::{OrderMetadata, OrderUid};
    use maplit::btreemap;
    use serde_json::json;

    #[test]
    fn roundtrips_auction() {
        let order = |uid_byte: u8| Order {
            metadata: OrderMetadata {
                uid: OrderUid([uid_byte; 56]),
                ..Default::default()
            },
            ..Default::default()
        };
        let auction = Auction {
            block: 42,
            latest_settlement_block: 40,
            orders: vec![order(1), order(2)],
            prices: btreemap! {
                H160([2; 20]) => U256::from(2),
                H160([1; 20]) => U256::from(1),
            },
        };
        let auction = AuctionWithId { id: 0, auction };

        assert_eq!(
            serde_json::to_value(&auction).unwrap(),
            json!({
                "id": 0,
                "block": 42,
                "latestSettlementBlock": 40,
                "orders": [
                    order(1),
                    order(2),
                ],
                "prices": {
                    "0x0101010101010101010101010101010101010101": "1",
                    "0x0202020202020202020202020202020202020202": "2",
                },
            }),
        );
        assert_eq!(
            serde_json::from_value::<AuctionWithId>(serde_json::to_value(&auction).unwrap())
                .unwrap(),
            auction,
        );
    }
}
