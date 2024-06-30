use frame_support::pallet_prelude::*;
use ark_bn254::{Fr};
use ark_ff::{PrimeField, BigInteger};
use crate::poll::{Commitment, Outcome, HashBytes, zeroes::get_merkle_zeroes};
use crate::hash::{Poseidon, PoseidonHasher, PoseidonError};

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub struct PollState
{
    /// The merkle tree of registration data.
    pub registrations: PollStateTree,

    /// The merkle tree of interaction data.
    pub interactions: PollStateTree,

    /// The current proof commitment.
    pub commitment: Commitment,

    /// The final result of the poll.
    pub outcome: Option<Outcome>,

    /// Whether the poll was nullified
    pub tombstone: bool
}

impl Default for PollState
{
    fn default() -> PollState
    {
        PollState {
            registrations: PollStateTree::new(2),
            interactions: PollStateTree::new(5),
            commitment: (0, [0; 32]),
            outcome: None,
            tombstone: false
        }
    }
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub struct PollStateTree
{
    /// The true depth of the tree (i.e., consisting of non-zero leaves).
    pub depth: u8,

    /// The arity of the tree.
    pub arity: u8,

    /// The number of non-nil leaves.
    pub count: u32,

    /// The (depth, hash) pairs of the incrementally merged subtrees.
    pub hashes: Vec<(u8, HashBytes)>,

    /// The root of the tree of maximal depth which contains the
    /// leaves of `hashes` and zeros elsewhere.
    pub root: Option<HashBytes>
}

pub enum MerkleTreeError
{
    /// The tree is full and cannot be inserted.
    TreeFull,
    /// The tree has already been merged.
    TreeMerged,
    /// The hash function did not succeed.
    HashFailed,
    // InvalidWidthCircom { width: usize, max_limit: usize },
}

pub trait AmortizedIncrementalMerkleTree: Sized
{
    /// The error type for the hash function.
    type HashError;

    /// Create a new tree.
    fn new(arity: u8) -> Self;

    /// Inserts a new leaf into the tree.
    fn insert(self, data: HashBytes) -> Result<Self, MerkleTreeError>;

    /// Compute the root of the tree.
    fn merge(self) -> Result<Self, MerkleTreeError>;

    /// Hash function used to compute roots.
    fn hash(inputs: Vec<HashBytes>) -> Result<HashBytes, Self::HashError>;
}

const FULL_TREE_DEPTH: u8 = 32;

impl AmortizedIncrementalMerkleTree for PollStateTree
{
    type HashError = PoseidonError;

    fn new(arity: u8) -> PollStateTree
    {
        PollStateTree {
            arity,
            depth: 0,
            count: 0,
            hashes: Vec::<(u8, HashBytes)>::new(),
            root: None
        }
    }

    /// Consumes a new leaf and produces the resultant partially merged merkle tree.
    ///
    /// -`leaf`: A new right-most leaf to insert into the tree.
    ///
    fn insert(
        mut self,
        leaf: HashBytes
    ) -> Result<Self, MerkleTreeError>
    {
        // Ensure that the tree is not full (or merged).
        if self.root != None { Err(MerkleTreeError::TreeFull)? }

        self.count += 1;
        self.hashes.push((0, leaf));

        let arity: usize = self.arity.into();

        loop
        {
            // We need at least `arity` nodes in order to compute a subtree root.
            let size = self.hashes.len();
            if size < arity { break; }

            let subtree = &self.hashes[self.hashes.len() - arity..];
            let depth = subtree[0].0;

            // If the subtree is full compute the corresponding subtree root.
            if subtree.iter().all(|&(d, _)| d == depth)
            {
                let leaves: Vec<HashBytes> = subtree
                    .iter()
                    .map(|&(_, hash)| hash)
                    .collect();

                let Some(hash) = Self::hash(leaves).ok() else { Err(MerkleTreeError::HashFailed)? };

                self.hashes.truncate(size - arity);
                self.hashes.push((depth + 1, hash));

                let true_depth = depth + 1; 
                if self.depth < true_depth { self.depth = true_depth; }
            }
            else { break; }
        }

        // If tree is full update the `root` property.
        if self.hashes.len() == 1 && self.hashes[0].0 == self.depth
        {
            self.root = Some(self.hashes[0].1);
            self.hashes.truncate(0);
        }

        Ok(self)
    }

    /// Obtain the root of the tree, wherein the remaining leaves take on zero values.
    /// NB we require the state tree to have a fixed height since the circuits must 
    /// know this value at compile time.
    fn merge(
        mut self
    ) -> Result<Self, MerkleTreeError>
    {
        // Ensure the tree is not already merged.
        if self.root != None { Err(MerkleTreeError::TreeMerged)? }

        let zeroes = get_merkle_zeroes(self.arity);
        if self.hashes.len() == 0
        {
            self.root = zeroes.last().copied();
            return Ok(self);
        }

        let arity: usize = self.arity.into();

        loop
        {
            let last = match self.hashes.last()
            {
                Some(&last) => last,
                None => break,
            };

            let depth = last.0;
            if depth >= FULL_TREE_DEPTH { break; }

            let mut subtree: Vec<_> = self.hashes
                .iter()
                .rev()
                .take_while(|(d, _)| *d == last.0)
                .cloned()
                .map(|(_, hash)| hash)
                .collect();
            
            let size = subtree.len();
            let zero = zeroes[depth as usize];
            if arity >= size { subtree .extend((0..(arity - size)).map(|_| zero)); }

            let Some(hash) = Self::hash(subtree).ok() else { Err(MerkleTreeError::HashFailed)? };
            self.hashes.truncate(self.hashes.len() - size);
            self.hashes.push((last.0 + 1, hash));
        }

        // Once tree is full update the `root` property.
        if self.hashes.len() == 1 && self.hashes[0].0 == self.depth
        {
            self.root = Some(self.hashes[0].1);
            self.hashes.truncate(0);
        }

        Ok(self)
    }

    /// Poseidon hash function with circom domain tag.
    fn hash(inputs: Vec<HashBytes>) -> Result<HashBytes, Self::HashError>
    {
        let mut hasher = Poseidon::<Fr>::new_circom(inputs.len())?;

        let fr_inputs: Vec<Fr> = inputs
            .iter()
            .map(|bytes| Fr::from_be_bytes_mod_order(bytes))
            .collect();

        let result = hasher
            .hash(&fr_inputs)?
            .into_bigint()
            .to_bytes_be();
        
        let mut bytes = [0u8; 32];
        bytes[..result.len()].copy_from_slice(&result);

        Ok(bytes)
    }
}