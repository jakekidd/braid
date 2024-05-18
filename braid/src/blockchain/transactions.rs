use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};
use secp256k1::{Message, Secp256k1, SecretKey, Signature};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone)]
pub struct State {
    pub player_address: String,
    pub move_hash: Vec<u8>,
    pub turn_number: u64,
}

#[derive(Clone)]
pub struct StateChannel {
    pub player_address: String,
    pub server_address: String,
    pub initial_state: State,
    pub current_state: State,
    pub player_signature: Option<Signature>,
    pub server_signature: Option<Signature>,
}

impl StateChannel {
    // Create a new state channel with an initial state.
    pub fn new(player_address: &str, server_address: &str) -> Self {
        let initial_state = State {
            player_address: player_address.to_string(),
            move_hash: vec![],
            turn_number: 0,
        };
        StateChannel {
            player_address: player_address.to_string(),
            server_address: server_address.to_string(),
            initial_state: initial_state.clone(),
            current_state: initial_state,
            player_signature: None,
            server_signature: None,
        }
    }

    // Sign the current state.
    pub fn sign_state(&mut self, secret_key: &SecretKey) -> Signature {
        let secp = Secp256k1::new();
        let state_bytes = bincode::serialize(&self.current_state).unwrap();
        let state_hash = Sha256::digest(&state_bytes);
        let message = Message::from_slice(&state_hash).unwrap();
        let sig = secp.sign(&message, secret_key);
        sig
    }

    // Update the state with a new move.
    pub fn update_state(&mut self, move_hash: Vec<u8>, turn_number: u64, player_signature: Signature, server_signature: Signature) {
        self.current_state = State {
            player_address: self.player_address.clone(),
            move_hash,
            turn_number,
        };
        self.player_signature = Some(player_signature);
        self.server_signature = Some(server_signature);
    }

    // Verify a signed state.
    pub fn verify_state(&self, state: &State, signature: &Signature, public_key: &secp256k1::PublicKey) -> bool {
        let secp = Secp256k1::new();
        let state_bytes = bincode::serialize(state).unwrap();
        let state_hash = Sha256::digest(&state_bytes);
        let message = Message::from_slice(&state_hash).unwrap();
        secp.verify(&message, signature, public_key).is_ok()
    }

    // Serialize the state for on-chain settlement.
    pub fn serialize_state(&self) -> Vec<u8> {
        bincode::serialize(&self.current_state).unwrap()
    }

    // Deserialize the state for on-chain settlement.
    pub fn deserialize_state(data: &[u8]) -> State {
        bincode::deserialize(data).unwrap()
    }
}

// Structure representing a blockchain transaction.
pub struct BlockchainTransaction {
    pub sender: String,
    pub receiver: String,
    pub amount: f64,
    pub data: Vec<u8>,
}

impl BlockchainTransaction {
    // Create a new blockchain transaction.
    pub fn new(sender: &str, receiver: &str, amount: f64, data: Vec<u8>) -> Self {
        BlockchainTransaction {
            sender: sender.to_string(),
            receiver: receiver.to_string(),
            amount,
            data,
        }
    }

    // Stub for committing an ante to the treasure pool.
    pub fn commit_ante(sender: &str, amount: f64) -> Self {
        BlockchainTransaction::new(sender, "treasure_pool", amount, vec![])
    }

    // Stub for submitting paths at the end of the game.
    pub fn submit_path(sender: &str, path: Vec<(usize, usize)>) -> Self {
        let data = bincode::serialize(&path).unwrap();
        BlockchainTransaction::new(sender, "game_contract", 0.0, data)
    }

    // Stub for claiming treasure if the player reached the center in time.
    pub fn claim_treasure(sender: &str, amount: f64) -> Self {
        BlockchainTransaction::new(sender, "treasure_pool", amount, vec![])
    }

    // Stub for slashing claims for misbehavior.
    pub fn slash_claim(sender: &str, receiver: &str, reason: &str) -> Self {
        let data = reason.as_bytes().to_vec();
        BlockchainTransaction::new(sender, receiver, 0.0, data)
    }

    // Stub for auditing transactions to ensure fair play.
    pub fn audit_transaction(sender: &str, data: Vec<u8>) -> Self {
        BlockchainTransaction::new(sender, "audit_contract", 0.0, data)
    }

    // Open a state channel.
    pub fn open_state_channel(sender: &str, receiver: &str, initial_state: State) -> Self {
        let data = bincode::serialize(&initial_state).unwrap();
        BlockchainTransaction::new(sender, receiver, 0.0, data)
    }

    // Close a state channel and settle on-chain.
    pub fn close_state_channel(sender: &str, receiver: &str, final_state: State) -> Self {
        let data = bincode::serialize(&final_state).unwrap();
        BlockchainTransaction::new(sender, receiver, 0.0, data)
    }

    // Commit a move on-chain in case of a dispute.
    pub fn commit_move_on_chain(sender: &str, move_hash: Vec<u8>, zk_proof: Vec<u8>) -> Self {
        let mut data = move_hash;
        data.extend(zk_proof);
        BlockchainTransaction::new(sender, "game_contract", 0.0, data)
    }

    // Example of how to serialize transaction data for sending to the blockchain.
    pub fn serialize(&self) -> Vec<u8> {
        bincode::serialize(self).unwrap()
    }

    // Example of how to deserialize transaction data received from the blockchain.
    pub fn deserialize(data: &[u8]) -> Self {
        bincode::deserialize(data).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_commit_ante() {
        let tx = BlockchainTransaction::commit_ante("player1", 100.0);
        assert_eq!(tx.sender, "player1");
        assert_eq!(tx.receiver, "treasure_pool");
        assert_eq!(tx.amount, 100.0);
    }

    #[test]
    fn test_submit_path() {
        let path = vec![(0, 0), (0, 1), (1, 1)];
        let tx = BlockchainTransaction::submit_path("player1", path.clone());
        assert_eq!(tx.sender, "player1");
        assert_eq!(tx.receiver, "game_contract");
        assert_eq!(tx.amount, 0.0);
        let data: Vec<(usize, usize)> = bincode::deserialize(&tx.data).unwrap();
        assert_eq!(data, path);
    }

    #[test]
    fn test_claim_treasure() {
        let tx = BlockchainTransaction::claim_treasure("player1", 500.0);
        assert_eq!(tx.sender, "player1");
        assert_eq!(tx.receiver, "treasure_pool");
        assert_eq!(tx.amount, 500.0);
    }

    #[test]
    fn test_slash_claim() {
        let tx = BlockchainTransaction::slash_claim("player1", "player2", "cheating");
        assert_eq!(tx.sender, "player1");
        assert_eq!(tx.receiver, "player2");
        assert_eq!(tx.amount, 0.0);
        assert_eq!(String::from_utf8(tx.data).unwrap(), "cheating");
    }

    #[test]
    fn test_audit_transaction() {
        let audit_data = vec![1, 2, 3, 4];
        let tx = BlockchainTransaction::audit_transaction("auditor", audit_data.clone());
        assert_eq!(tx.sender, "auditor");
        assert_eq!(tx.receiver, "audit_contract");
        assert_eq!(tx.amount, 0.0);
        assert_eq!(tx.data, audit_data);
    }

    #[test]
    fn test_open_state_channel() {
        let initial_state = State {
            player_address: "player1".to_string(),
            move_hash: vec![0, 1, 2, 3],
            turn_number: 0,
        };
        let tx = BlockchainTransaction::open_state_channel("player1", "server1", initial_state.clone());
        assert_eq!(tx.sender, "player1");
        assert_eq!(tx.receiver, "server1");
        assert_eq!(tx.amount, 0.0);
        let data: State = bincode::deserialize(&tx.data).unwrap();
        assert_eq!(data, initial_state);
    }

    #[test]
    fn test_close_state_channel() {
        let final_state = State {
            player_address: "player1".to_string(),
            move_hash: vec![0, 1, 2, 3],
            turn_number: 10,
        };
        let tx = BlockchainTransaction::close_state_channel("player1", "server1", final_state.clone());
        assert_eq!(tx.sender, "player1");
        assert_eq!(tx.receiver, "server1");
        assert_eq!(tx.amount, 0.0);
        let data: State = bincode::deserialize(&tx.data).unwrap();
        assert_eq!(data, final_state);
    }

    #[test]
    fn test_commit_move_on_chain() {
        let move_hash = vec![0, 1, 2, 3];
        let zk_proof = vec![4, 5, 6, 7];
        let tx = BlockchainTransaction::commit_move_on_chain("player1", move_hash.clone(), zk_proof.clone());
        assert_eq!(tx.sender, "player1");
        assert_eq!(tx.receiver, "game_contract");
        assert_eq!(tx.amount, 0.0);
        assert_eq!(tx.data[..move_hash.len()], move_hash[..]);
        assert_eq!(tx.data[move_hash.len()..], zk_proof[..]);
    }
}
