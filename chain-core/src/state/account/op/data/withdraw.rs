use crate::init::coin::{sum_coins, Coin, CoinError};
use crate::state::account::Nonce;
use crate::tx::data::attribute::TxAttributes;
use crate::tx::data::output::TxOut;
#[cfg(feature = "new-txid")]
use crate::tx::TaggedTransaction;
#[cfg(not(feature = "new-txid"))]
use crate::tx::TransactionId;
use parity_scale_codec::{Decode, Encode, Error, Input, Output};

use serde::{Deserialize, Serialize};

use std::fmt;
use std::prelude::v1::Vec;

/// takes the StakedState (implicit from the witness) and creates UTXOs
/// (update's StakedState's unbonded + nonce)
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct WithdrawUnbondedTx {
    /// counter to check against
    pub nonce: Nonce,
    /// new outputs to create
    pub outputs: Vec<TxOut>,
    /// view policy, versining info etc.
    pub attributes: TxAttributes,
}

impl Decode for WithdrawUnbondedTx {
    fn decode<I: Input>(input: &mut I) -> Result<Self, Error> {
        let nonce = Nonce::decode(input)?;
        let outputs: Vec<TxOut> = Vec::decode(input)?;
        let attributes = TxAttributes::decode(input)?;

        Ok(WithdrawUnbondedTx {
            nonce,
            outputs,
            attributes,
        })
    }
}

impl Encode for WithdrawUnbondedTx {
    fn encode_to<EncOut: Output>(&self, dest: &mut EncOut) {
        dest.push(&self.nonce);
        dest.push(&self.outputs);
        dest.push(&self.attributes);
    }

    fn size_hint(&self) -> usize {
        self.outputs.size_hint() + self.nonce.size_hint() + self.attributes.size_hint()
    }
}

#[cfg(not(feature = "new-txid"))]
impl TransactionId for WithdrawUnbondedTx {}

#[cfg(feature = "new-txid")]
impl From<WithdrawUnbondedTx> for TaggedTransaction {
    fn from(tx: WithdrawUnbondedTx) -> TaggedTransaction {
        TaggedTransaction::Withdraw(tx)
    }
}

impl WithdrawUnbondedTx {
    /// creates a new tx to withdraw unbonded stake
    pub fn new(nonce: Nonce, outputs: Vec<TxOut>, attributes: TxAttributes) -> Self {
        WithdrawUnbondedTx {
            nonce,
            outputs,
            attributes,
        }
    }
}

impl WithdrawUnbondedTx {
    /// returns the total transaction output amount (sum of all output amounts)
    pub fn get_output_total(&self) -> Result<Coin, CoinError> {
        sum_coins(self.outputs.iter().map(|x| x.value))
    }
}

impl fmt::Display for WithdrawUnbondedTx {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "-> (unbonded) (nonce: {})", self.nonce)?;
        for output in self.outputs.iter() {
            writeln!(f, "   {} ->", output)?;
        }
        write!(f, "")
    }
}
