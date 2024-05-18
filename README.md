# Braid

A dungeon crawler game using a P2P state channel network and ZK. Dungeon nodes supply Players with the maze, and Players race to navigate, discovering and recording new parts of the maze as they go. At the center of the maze is a treasure comprised of the Players' antes for the game.

## Use of Zero-Knowledge Proofs

### Proof of Honest Play

You can use ZK proofs to ensure that players are making legitimate moves without revealing their exact position in the maze. Each player would generate a ZK proof that validates they have moved according to the rules from their last known state, without actually disclosing their new position in the maze. This prevents cheating while maintaining privacy.

#### Implementation:

1. Cryptographic Commitments: At the start of each turn, a player's current position is committed using a cryptographic hash function (e.g., SHA-256). This commitment is published on-chain or through state channels.
2. ZK-SNARKs or ZK-STARKs: When a player makes a move, they generate a zero-knowledge proof (either SNARK or STARK) that validates the move from the previously committed position to the new position without revealing either position.
3. Circuit Construction:
    a. Input: The hash of the previous position, the new position, and the action taken.
    b. Output: A boolean indicating the legality of the move.
    c. Verification: The game smart contract verifies the proof against the public commitment of the previous position before accepting the new position.

### Proof of Correct Maze Generation
Ensure the integrity of the maze itself by requiring Dungeon nodes to generate a ZK proof that the maze they create is solvable from the start. This proof would be checked when the maze is first submitted to the blockchain, ensuring all players have a fair chance to solve the maze.

#### Implementation:

1. Generate the Maze: The maze generation algorithm outputs a maze along with a solution path from the start to the treasure.
2. Cryptographic Commitment: The solution path is committed cryptographically and stored without revealing it.
3. ZK-SNARKs or ZK-STARKs: Generate a ZK proof that there exists a valid solution path from the starting point to the treasure without revealing the path.
4. Circuit Construction:
    a. Input: The start and end points, and the maze structure.
    b. Output: A boolean that validates the existence of a solution path.
    c. Verification: This proof is submitted along with the maze data to the blockchain or through state channels for verification.

### Proof of Non-Collusion
Implement a mechanism where players can generate ZK proofs to show that they haven’t shared their explored areas with other players. This could be tricky but might involve commitments made at the beginning of the game that are checked against during or after gameplay.

#### Implementation:

1. Commitment at Game Start: At the beginning of the game, each player generates cryptographic commitments of their exploration strategy or path, which is recorded on the blockchain.
2. Periodic Updates: As players explore new areas of the maze, they generate and submit ZK proofs that verify they are following the committed strategy or exploring independently.
3. Circuit Construction:
a. Input: Player's current path, committed path, and any updates made during gameplay.
b. Output: A boolean that checks consistency with the initial commitment without revealing the path.
c. Verification: These proofs can be checked by other players or the game contract to ensure no information sharing or collusion has occurred.

## Sybil Resistance

### Registration System
Implement a registration system with identity verification to prevent multiple accounts per player. This could be done through a third-party service that links Ethereum addresses to verified identities without compromising the anonymity within the game. Helps reduce Sybil attack vulnerability.

### Randomized Pairing of Players and Dungeons
Use a verifiable random function (VRF) to assign players to dungeons. This helps ensure the fairness and unpredictability of assignments, reducing the chance of pre-game collusion. Also helps reduce Sybil attack vulnerability.

## Game Theory

### Expiring Treasure
The Dungeon nodes have an incentive to only share the spaces the Player has explored so far with each Player: the treasure decreases over time, and as it decreases, the share the Dungeon node keeps for the game as a fee increases. The Dungeons, therefore, want the Players to fail.

### Collaboration
Each Player has an incentive to not share what they’ve explored so far with other Players, since they will potentially lose out on share of the treasure. Friends (or multiple addresses belonging to the same Sybil attacker) could still end up in the same game and cooperate together. Maybe we’ll come back to that.

### Solvability Requirement
The Dungeons have an incentive to produce a maze that’s solvable. They have to stake beforehand, and must prove at the end that the maze is solvable. This can be done by ending the game, perhaps after a designated number of turns, with simply providing the solving path.

## On-Chain Accountability

### Audit Trails and Public Verification
We keep an encrypted log of moves and game events on-chain that can be publicly verified post-game. In the case of disputes, these logs can be decrypted in a controlled manner to validate claims without revealing more information than necessary.

### Continuous Challenge Mechanism
Allow players to challenge the state of the game at any point, not only when they suspect wrongdoing.

`isAlive` failure: If the Dungeon node is not producing new data, the Players should be able to challenge. We could use ZK for this, but a simple (separate) private/public ECDH key interaction would solve this I think? If the Player submits a challenge saying the Dungeon node is not producing dungeon data, the Dungeon node needs to be able to go to chain and deliver a section of the dungeon corresponding to the Player’s move, but not reveal that section to everybody. It’s possible we should just do a revealing proof - an audit trail - after the game is over, where the Player reveals, for example, a private key or something? The Dungeon should be able to prove at some point that they’re handing out information and doing so correctly. If the Player lies on-chain about their moves, the Dungeon should have the Player’s state channel signature on their actual moves that they can use to counter the challenge.

## Implementation Considerations

### Scalability
We need to ensure that the on-chain interactions (like posting ZK proofs or handling disputes) are scalable. This could involve layer-2 solutions or state channels to handle the frequent updates and proofs required by the game.

### User Experience
Focus on making the game accessible and enjoyable. Even though the underlying technology is complex, the player interface should be simple and intuitive.
