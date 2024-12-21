use std::collections::{HashSet, VecDeque};
use std::iter::Chain;
use std::slice::Iter;

/// Unique identifier for block hashes
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
struct BlockHash(u64);

impl BlockHash {
    fn new() -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        BlockHash(COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

/// Transaction output
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
struct TXO {
    tx_id: u64,  // Reference to parent transaction
    index: usize,
    value: u64,
}

/// Shielded note
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
struct Note {
    id: u64,    // Unique identifier
    value: u64,
}

impl Note {
    fn new(value: u64) -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        Note {
            id: COUNTER.fetch_add(1, Ordering::Relaxed),
            value,
        }
    }
}

#[derive(Debug, Clone)]
struct BCTransaction {
    transparent_inputs: Vec<TXO>,
    transparent_outputs: Vec<TXO>,
    shielded_inputs: Vec<Note>,
    shielded_outputs: Vec<Note>,
    fee: i64,
    anchor: Option<BCContext>,
    issuance: u64,
    id: u64,
}

impl BCTransaction {
    fn new(
        transparent_inputs: Vec<TXO>,
        transparent_output_values: Vec<u64>,
        shielded_inputs: Vec<Note>,
        shielded_output_values: Vec<u64>,
        fee: i64,
        anchor: Option<BCContext>,
        issuance: u64,
    ) -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let id = COUNTER.fetch_add(1, Ordering::Relaxed);
        
        let transparent_outputs = transparent_output_values
            .iter()
            .enumerate()
            .map(|(i, v)| TXO { tx_id: id, index: i, value: *v })
            .collect();

        let shielded_outputs = shielded_output_values
            .iter()
            .map(|v| Note::new(*v))
            .collect();

        // Validate transaction
        assert!(issuance >= 0);
        let is_coinbase = transparent_inputs.is_empty() && shielded_inputs.is_empty();
        assert!(fee >= 0 || is_coinbase);
        assert!(issuance == 0 || is_coinbase);

        let total_in: u64 = transparent_inputs.iter().map(|txo| txo.value).sum::<u64>() +
                        shielded_inputs.iter().map(|note| note.value).sum::<u64>() +
                        issuance;
        
        let total_out: u64 = transparent_output_values.iter().sum::<u64>() +
                            shielded_output_values.iter().sum::<u64>() +
                            if fee >= 0 { fee as u64 } else { 0 };

        assert_eq!(total_in, total_out);

        BCTransaction {
            transparent_inputs,
            transparent_outputs,
            shielded_inputs,
            shielded_outputs,
            fee,
            anchor,
            issuance,
            id,
        }
    }

    fn is_coinbase(&self) -> bool {
        self.transparent_inputs.is_empty() && self.shielded_inputs.is_empty()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Spentness {
    Unspent,
    Spent,
}

#[derive(Debug, Clone)]
struct BCContext {
    transactions: VecDeque<BCTransaction>,
    utxo_set: HashSet<TXO>,
    notes: Vec<(Note, Spentness)>,
    total_issuance: u64,
}

impl BCContext {
    fn new() -> Self {
        BCContext {
            transactions: VecDeque::new(),
            utxo_set: HashSet::new(),
            notes: Vec::new(),
            total_issuance: 0,
        }
    }

    fn can_spend(&self, to_spend: &[Note]) -> bool {
        to_spend.iter().all(|note| {
            self.notes.iter()
                .find(|(n, _)| n == note)
                .map_or(false, |(_, status)| *status == Spentness::Unspent)
        })
    }

    fn is_valid(&self, tx: &BCTransaction) -> bool {
        // TODO: Check if I really need to clone the TXOs for a simple check
        let tx_inputs: HashSet<TXO> = tx.transparent_inputs.iter().cloned().collect();
        tx_inputs.is_subset(&self.utxo_set) && self.can_spend(&tx.shielded_inputs)
    }

    fn add_if_valid(&mut self, tx: BCTransaction) -> bool {
        if !self.is_valid(&tx) {
            return false;
        }

        // Remove spent UTXOs and add new ones
        for input in &tx.transparent_inputs {
            self.utxo_set.remove(input);
        }
        self.utxo_set.extend(tx.transparent_outputs.clone());

        // Update note spentness
        for input in &tx.shielded_inputs {
            if let Some(pos) = self.notes.iter().position(|(n, _)| n == input) {
                self.notes[pos].1 = Spentness::Spent;
            }
        }
        
        for output in &tx.shielded_outputs {
            self.notes.push((output.clone(), Spentness::Unspent));
        }

        self.total_issuance += tx.issuance;
        self.transactions.push_back(tx);
        true
    }
}

#[derive(Debug)]
struct BCBlock {
    parent: Option<Box<BCBlock>>,
    score: i64,
    transactions: Vec<BCTransaction>,
    hash: BlockHash,
}

impl BCBlock {
    fn new(
        parent: Option<BCBlock>,
        added_score: i64,
        transactions: Vec<BCTransaction>,
        allow_invalid: bool,
    ) -> Self {
        let parent = parent.map(Box::new);
        let score = parent.as_ref().map_or(added_score, |p| p.score + added_score);
        let hash = BlockHash::new();

        let block = BCBlock {
            parent,
            score,
            transactions,
            hash,
        };

        if !allow_invalid {
            block.assert_noncontextually_valid();
        }
        block
    }

    fn assert_noncontextually_valid(&self) {
        assert!(!self.transactions.is_empty());
        assert!(self.transactions[0].is_coinbase());
        assert!(self.transactions[1..].iter().all(|tx| !tx.is_coinbase()));
        assert_eq!(self.transactions.iter().map(|tx| tx.fee).sum::<i64>(), 0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        let mut ctx = BCContext::new();
        
        // Genesis block
        let coinbase_tx0 = BCTransaction::new(vec![], vec![10], vec![], vec![], 0, None, 10);
        assert!(ctx.add_if_valid(coinbase_tx0.clone()));
        let genesis = BCBlock::new(None, 1, vec![coinbase_tx0.clone()], false);
        assert_eq!(genesis.score, 1);
        assert_eq!(ctx.total_issuance, 10);

        // More tests can be added following the Python test pattern...
    }
}