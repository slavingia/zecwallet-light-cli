use crate::{
    compact_formats::{CompactBlock, CompactOutput, CompactTx},
    lightwallet::data::BlockData,
};
use prost::Message;
use rand::{rngs::OsRng, RngCore};
use zcash_primitives::{
    block::BlockHash,
    merkle_tree::{CommitmentTree, Hashable, IncrementalWitness},
    primitives::{Note, Rseed},
    sapling::Node,
    transaction::components::Amount,
    zip32::{ExtendedFullViewingKey, ExtendedSpendingKey},
};

pub fn random_u8_32() -> [u8; 32] {
    let mut b = [0u8; 32];
    OsRng.fill_bytes(&mut b);

    b
}

pub fn tree_to_string(tree: &CommitmentTree<Node>) -> String {
    let mut b1 = vec![];
    tree.write(&mut b1).unwrap();
    hex::encode(b1)
}

pub fn incw_to_string(inc_witness: &IncrementalWitness<Node>) -> String {
    let mut b1 = vec![];
    inc_witness.write(&mut b1).unwrap();
    hex::encode(b1)
}

pub fn node_to_string(n: &Node) -> String {
    let mut b1 = vec![];
    n.write(&mut b1).unwrap();
    hex::encode(b1)
}

pub fn list_all_witness_nodes(cb: &CompactBlock) -> Vec<Node> {
    let mut nodes = vec![];
    for tx in &cb.vtx {
        for co in &tx.outputs {
            nodes.push(Node::new(co.cmu().unwrap().into()))
        }
    }

    nodes
}

pub struct FakeCompactBlock {
    pub block: CompactBlock,
}

impl FakeCompactBlock {
    pub fn new(height: u64, prev_hash: BlockHash) -> Self {
        // Create a fake Note for the account
        let mut rng = OsRng;

        let mut cb = CompactBlock::default();

        cb.height = height;
        cb.hash.resize(32, 0);
        rng.fill_bytes(&mut cb.hash);

        cb.prev_hash.extend_from_slice(&prev_hash.0);

        Self { block: cb }
    }

    // Add a new tx into the block, paying the given address the amount.
    // Returns the nullifier of the new note.
    pub fn add_random_outputs(&mut self, num_outputs: usize) {
        let xsk_m = ExtendedSpendingKey::master(&[1u8; 32]);
        let extfvk = ExtendedFullViewingKey::from(&xsk_m);

        let to = extfvk.default_address().unwrap().1;
        let value = Amount::from_u64(1).unwrap();

        let mut ctx = CompactTx::default();
        ctx.hash = random_u8_32().to_vec();

        for _ in 0..num_outputs {
            // Create a fake Note for the account
            let note = Note {
                g_d: to.diversifier().g_d().unwrap(),
                pk_d: to.pk_d().clone(),
                value: value.into(),
                rseed: Rseed::AfterZip212(random_u8_32()),
            };

            // Create a fake CompactBlock containing the note
            let mut cout = CompactOutput::default();
            cout.cmu = note.cmu().to_bytes().to_vec();

            ctx.outputs.push(cout);
        }

        self.block.vtx.push(ctx);
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let mut b = vec![];
        self.block.encode(&mut b).unwrap();

        b
    }

    pub fn into(self) -> CompactBlock {
        self.block
    }
}

pub struct FakeCompactBlockList {
    pub blocks: Vec<FakeCompactBlock>,
}

impl FakeCompactBlockList {
    pub fn new(len: u64) -> Self {
        let mut blocks = vec![];
        let mut prev_hash = BlockHash([0u8; 32]);

        for i in 0..len {
            let mut b = FakeCompactBlock::new(i + 1, prev_hash);
            prev_hash = b.block.hash();

            // Add 2 transactions, each with some random Compact Outputs to this block
            for _ in 0..2 {
                b.add_random_outputs(2);
            }

            blocks.push(b);
        }

        Self { blocks }
    }

    pub fn into(self) -> Vec<BlockData> {
        self.blocks
            .into_iter()
            .map(|fcb| BlockData::new(fcb.into()))
            .rev()
            .collect()
    }
}
