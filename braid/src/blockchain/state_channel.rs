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

// Example of using secp256k1 for signing and verifying.
fn example_usage() {
    let secp = Secp256k1::new();
    let (secret_key, public_key) = secp.generate_keypair(&mut rand::thread_rng());

    let mut channel = StateChannel::new("player_address", "server_address");
    let move_hash = vec![0, 1, 2, 3];
    let turn_number = 1;

    // Player signs the state.
    let player_sig = channel.sign_state(&secret_key);

    // Server updates the state and signs it.
    channel.update_state(move_hash.clone(), turn_number, player_sig.clone(), player_sig.clone());

    // Verify the state.
    assert!(channel.verify_state(&channel.current_state, &player_sig, &public_key));
}
