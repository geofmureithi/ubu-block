use sha3::{Digest, Sha3_256 as Sha256};
use crate::CandidateResult;

#[derive(Debug, Clone)]
pub struct MerkleNode {
    pub hash: [u8; 32],
    pub left: Option<Box<MerkleNode>>,
    pub right: Option<Box<MerkleNode>>,
}

pub struct MerkleTree {
    pub root: Option<Box<MerkleNode>>,
    pub leaves: Vec<[u8; 32]>,
}

impl MerkleTree {
    pub fn new() -> Self {
        Self {
            root: None,
            leaves: Vec::new(),
        }
    }

    // Build Merkle tree from election results
    pub fn from_election_results(results: &[CandidateResult]) -> Self {
        let mut tree = Self::new();

        // Step 1: Hash each election result to create leaves
        for result in results {
            let result_hash = Self::hash_election_result(result);
            tree.leaves.push(result_hash);
        }

        // Step 2: Build the tree from leaves up to root
        if !tree.leaves.is_empty() {
            tree.root = Some(Box::new(Self::build_tree(&tree.leaves)));
        }

        tree
    }

    // Hash a single election result
    fn hash_election_result(result: &CandidateResult) -> [u8; 32] {
        let serialized = bincode::serialize(result).unwrap();
        let mut hasher = Sha256::new();
        hasher.update(&serialized);
        hasher.finalize().into()
    }

    // Recursively build the Merkle tree
    fn build_tree(hashes: &[[u8; 32]]) -> MerkleNode {
        // Base case: single hash becomes a leaf node
        if hashes.len() == 1 {
            return MerkleNode {
                hash: hashes[0],
                left: None,
                right: None,
            };
        }

        // Recursive case: pair up hashes and build next level
        let mut next_level = Vec::new();

        for chunk in hashes.chunks(2) {
            match chunk.len() {
                2 => {
                    // Normal case: hash two nodes together
                    let combined_hash = Self::hash_pair(chunk[0], chunk[1]);
                    next_level.push(combined_hash);
                }
                1 => {
                    // Odd number of nodes: duplicate the last one
                    let combined_hash = Self::hash_pair(chunk[0], chunk[0]);
                    next_level.push(combined_hash);
                }
                _ => unreachable!(),
            }
        }

        // Build parent node
        let _parent_hash = next_level[next_level.len() - 1]; // This will be overwritten
        let mut node = MerkleNode {
            hash: [0; 32], // Temporary
            left: None,
            right: None,
        };

        // If we're not at the root yet, recursively build upper levels
        if next_level.len() > 1 {
            Self::build_tree(&next_level)
        } else {
            // We're at the root
            node.hash = next_level[0];

            // Build the actual tree structure with children
            if hashes.len() >= 2 {
                let mid = hashes.len().div_ceil(2);
                node.left = Some(Box::new(Self::build_tree(&hashes[..mid])));
                if mid < hashes.len() {
                    node.right = Some(Box::new(Self::build_tree(&hashes[mid..])));
                } else {
                    node.right = node.left.clone(); // Duplicate for odd numbers
                }
            }

            node
        }
    }

    // Better implementation that properly tracks tree structure
    fn build_tree_proper(hashes: &[[u8; 32]]) -> MerkleNode {
        if hashes.len() == 1 {
            return MerkleNode {
                hash: hashes[0],
                left: None,
                right: None,
            };
        }

        let mut current_level: Vec<MerkleNode> = hashes
            .iter()
            .map(|&hash| MerkleNode {
                hash,
                left: None,
                right: None,
            })
            .collect();

        // Build tree bottom-up
        while current_level.len() > 1 {
            let mut next_level = Vec::new();

            for chunk in current_level.chunks(2) {
                let (left, right) = match chunk.len() {
                    2 => (chunk[0].clone(), chunk[1].clone()),
                    1 => (chunk[0].clone(), chunk[0].clone()), // Duplicate for odd numbers
                    _ => unreachable!(),
                };

                let combined_hash = Self::hash_pair(left.hash, right.hash);
                let parent = MerkleNode {
                    hash: combined_hash,
                    left: Some(Box::new(left)),
                    right: Some(Box::new(right)),
                };

                next_level.push(parent);
            }

            current_level = next_level;
        }

        current_level.into_iter().next().unwrap()
    }

    // Create new tree using the proper implementation
    pub fn from_election_results_proper(results: &[CandidateResult]) -> Self {
        let mut tree = Self::new();

        for result in results {
            let result_hash = Self::hash_election_result(result);
            tree.leaves.push(result_hash);
        }

        if !tree.leaves.is_empty() {
            tree.root = Some(Box::new(Self::build_tree_proper(&tree.leaves)));
        }

        tree
    }

    // Hash two values together
    fn hash_pair(left: [u8; 32], right: [u8; 32]) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(left);
        hasher.update(right);
        hasher.finalize().into()
    }

    // Get the Merkle root hash
    pub fn get_root_hash(&self) -> Option<[u8; 32]> {
        self.root.as_ref().map(|node| node.hash)
    }

    // Generate a Merkle proof for a specific election result
    pub fn generate_proof(&self, target_index: usize) -> Option<Vec<[u8; 32]>> {
        if target_index >= self.leaves.len() {
            return None;
        }

        let mut proof = Vec::new();
        Self::generate_proof_recursive(&self.root, target_index, 0, self.leaves.len(), &mut proof);
        Some(proof)
    }

    fn generate_proof_recursive(
        node: &Option<Box<MerkleNode>>,
        target_index: usize,
        start_index: usize,
        end_index: usize,
        proof: &mut Vec<[u8; 32]>,
    ) -> bool {
        if let Some(node) = node {
            // Leaf node
            if node.left.is_none() && node.right.is_none() {
                return start_index == target_index;
            }

            let mid = (start_index + end_index).div_ceil(2);

            if target_index < mid {
                // Target is in left subtree
                if let Some(right) = &node.right {
                    proof.push(right.hash);
                }
                return Self::generate_proof_recursive(
                    &node.left,
                    target_index,
                    start_index,
                    mid,
                    proof,
                );
            } else {
                // Target is in right subtree
                if let Some(left) = &node.left {
                    proof.push(left.hash);
                }
                return Self::generate_proof_recursive(
                    &node.right,
                    target_index,
                    mid,
                    end_index,
                    proof,
                );
            }
        }
        false
    }

    // Verify a Merkle proof
    pub fn verify_proof(
        leaf_hash: [u8; 32],
        proof: &[[u8; 32]],
        root_hash: [u8; 32],
        leaf_index: usize,
        total_leaves: usize,
    ) -> bool {
        let mut current_hash = leaf_hash;
        let mut index = leaf_index;
        let mut range_size = total_leaves;

        for &sibling_hash in proof {
            range_size = range_size.div_ceil(2);
            if index % 2 == 0 {
                // Current node is left child
                current_hash = Self::hash_pair(current_hash, sibling_hash);
            } else {
                // Current node is right child
                current_hash = Self::hash_pair(sibling_hash, current_hash);
            }
            index /= 2;
        }

        current_hash == root_hash
    }

    // Pretty print the tree structure
    pub fn print_tree(&self) {
        if let Some(root) = &self.root {
            Self::print_node(root, 0);
        }
    }

    fn print_node(node: &MerkleNode, depth: usize) {
        let indent = "  ".repeat(depth);
        println!("{}Hash: {:x?}...", indent, &node.hash[..4]);

        if let Some(left) = &node.left {
            println!("{indent}├─ Left:");
            Self::print_node(left, depth + 1);
        }

        if let Some(right) = &node.right {
            println!("{indent}└─ Right:");
            Self::print_node(right, depth + 1);
        }
    }
}

#[cfg(test)]
mod tests {
    use types::CandidateResult;

    use super::*;

    #[test]
    fn test_merkle_tree_single_item() {
        let results = vec![CandidateResult {
            station_id: 1,
            candidate_id: 1,
            votes: 100,
        }];

        let tree = MerkleTree::from_election_results_proper(&results);
        assert!(tree.get_root_hash().is_some());
    }

    #[test]
    fn test_merkle_tree_multiple_items() {
        let mut results = Vec::new();
        for i in 0..4 {
            results.push(CandidateResult {
                station_id: i as i64,
                candidate_id: (i + 1) as i64,
                votes: (100 + i) as i64,
            });
        }

        let tree = MerkleTree::from_election_results_proper(&results);
        let root_hash = tree.get_root_hash().unwrap();

        // Test proof generation and verification
        let proof = tree.generate_proof(1).unwrap();
        let leaf_hash = MerkleTree::hash_election_result(&results[1]);

        assert!(MerkleTree::verify_proof(
            leaf_hash,
            &proof,
            root_hash,
            1,
            results.len()
        ));
    }

    #[test]
    fn test_merkle_proof_verification() {
        // Test with odd number of items
        let mut results = Vec::new();
        for i in 0..5 {
            results.push(CandidateResult {
                station_id: i as i64,
                candidate_id: (i + 1) as i64,
                votes: (100 + i) as i64,
            });
        }

        let tree = MerkleTree::from_election_results_proper(&results);
        let root_hash = tree.get_root_hash().unwrap();

        // Verify proofs for all items
        for i in 0..results.len() {
            let proof = tree.generate_proof(i).unwrap();
            let leaf_hash = MerkleTree::hash_election_result(&results[i]);

            assert!(
                MerkleTree::verify_proof(leaf_hash, &proof, root_hash, i, results.len()),
                "Proof verification failed for item {i}"
            );
        }
    }

    #[test]
    fn test_merkle_tree_example() {
        // Create sample election results
        let results = vec![
            CandidateResult {
                station_id: 1,
                candidate_id: 1,
                votes: 150,
            },
            CandidateResult {
                station_id: 2,
                candidate_id: 2,
                votes: 200,
            },
            CandidateResult {
                station_id: 3,
                candidate_id: 1,
                votes: 250,
            },
            CandidateResult {
                station_id: 4,
                candidate_id: 3,
                votes: 300,
            },
            CandidateResult {
                station_id: 5,
                candidate_id: 2,
                votes: 350,
            },
        ];

        // Build Merkle tree
        let tree = MerkleTree::from_election_results_proper(&results);

        println!("Merkle Tree Structure:");
        tree.print_tree();

        if let Some(root_hash) = tree.get_root_hash() {
            println!("\nMerkle Root: {root_hash:x?}");

            // Generate and verify proof for the first result
            if let Some(proof) = tree.generate_proof(0) {
                println!("\nProof for STATION_001:");
                for (i, hash) in proof.iter().enumerate() {
                    println!("  Step {}: {:x?}...", i, &hash[..8]);
                }

                // Verify the proof
                let leaf_hash = MerkleTree::hash_election_result(&results[0]);
                let is_valid =
                    MerkleTree::verify_proof(leaf_hash, &proof, root_hash, 0, results.len());

                println!(
                    "Proof verification: {}",
                    if is_valid { "VALID" } else { "INVALID" }
                );
            }
        }
    }
}
