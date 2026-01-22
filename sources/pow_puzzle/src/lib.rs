use sha2::{Digest, Sha256};

pub type NodeId = Vec<u8>;

pub fn check_static_puzzle(public_key: &Vec<u8>, complexity: usize) -> Option<NodeId> {
    let p = Sha256::digest(Sha256::digest(public_key));

    for digit in p.into_iter().take(complexity) {
        if digit != 0 {
            return None;
        }
    }

    let node_id = Sha256::digest(public_key);
    Some(node_id.into_iter().collect())
}

pub fn check_dynamic_puzzle(node_id: &NodeId, solution: &Vec<u8>, complexity: usize) -> bool {
    let xor: Vec<u8> = node_id
        .iter()
        .zip(solution.iter())
        .map(|(&x1, &x2)| x1 ^ x2)
        .collect();

    let p = Sha256::digest(xor);

    for digit in p.into_iter().take(complexity) {
        if digit != 0 {
            return false;
        }
    }

    true
}
