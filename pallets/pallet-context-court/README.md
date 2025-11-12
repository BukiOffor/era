# Context Court Pallet for Substrate

## Project Overview

This project implements a **Context Court Pallet** for Substrate, designed to facilitate decentralized dispute resolution for content context. The pallet enables a jury-based system where registered jurors can vote on disputes regarding the context or meaning of verified content. The pallet provides a two-tier voting system: an initial jury session and an escalated session involving all registered jurors when consensus cannot be reached. The design prioritizes game-theoretic security through economic incentives, slashing jurors who fail to participate and rewarding those who vote correctly. The consequence of this design is that disputes are resolved through decentralized human judgment, creating a "human oracle" system that bridges the gap between cryptographic content verification and contextual truth.

## Background and Considerations

While cryptographic proofs can verify content authenticity, they cannot verify context or meaning. A real photo from 2018 might be posted with the claim "This is happening right now." The Context Court pallet addresses this by creating a decentralized dispute resolution system. Key considerations included:

- **Jury Selection**: Random selection of jurors from a registered pool ensures fairness and prevents gaming.

- **Economic Security**: Jurors must stake tokens to register, creating economic commitment and enabling slashing for misbehavior.

- **Escalation Mechanism**: When initial jury votes are tied or inconclusive, disputes escalate to all registered jurors for a final decision.

- **Batch Processing**: Rewards and slashes are processed in batches during block initialization to manage gas costs and prevent DoS attacks.

- **Temporal Constraints**: Disputes have expiration blocks, preventing indefinite storage of unresolved disputes.

- **Vote Types**: Jurors can vote Yay (convict), Nay (acquit), or Abstain, providing flexibility in decision-making.

- **No Reward for Escalation**: Escalated sessions do not provide rewards, only slashes, creating an incentive to resolve disputes in the initial session.

## State Transition Function Design

The core logic of the pallet is structured as follows:

1. **Juror Registration**:

   - A user with `Dispute` rights for a DID can register as a juror.

   - The DID is added to the jurors pool.

   - A deposit is held from the user's account to prevent spam and enable slashing.

   - The user's account is mapped to the DID for reward/slash distribution.

2. **Dispute Creation**:

   - A user with `Dispute` rights creates a dispute for a specific `ContentId`.

   - A `CourtSession` is created with:
     - Empty jurors list (to be populated)
     - Context description (bounded bytes)
     - Expiration block number
     - Pending verdict

   - Jurors are automatically summoned using randomness.

3. **Jury Summoning**:

   - The system uses on-chain randomness to select jurors from the pool.

   - Selected jurors are added to `JurySelection` storage.

   - Jurors are tracked in `JuryDuty` to prevent duplicate selection.

   - The number of summoned jurors is tracked in `JurySummoned`.

   - Summoning continues until minimum jurors are reached or maximum is hit.

4. **Deliberation Start**:

   - Once minimum jurors are summoned, deliberation can begin.

   - The selected jurors are moved from `JurySelection` to the session's jurors list.

   - The session's `started_at` block is recorded.

   - The session must start before the expiration block.

5. **Voting**:

   - Jurors in an active session can cast votes (Yay, Nay, or Abstain).

   - Each juror can only vote once per session.

   - Votes are recorded in the session's verdict.

   - Voting can continue until the session expires.

6. **Result Calculation**:

   - After expiration, the result can be calculated.

   - If votes are tied (50/50), the dispute escalates.

   - Otherwise, a decision (Convict or Acquittal) is recorded.

   - Escalated disputes move to `EscalatedSession` storage.

7. **Escalated Voting**:

   - All registered jurors can vote on escalated disputes.

   - Escalated sessions have their own voting period.

   - After the escalated period, a final decision is calculated.

8. **Reward and Slash Distribution**:

   - After a session ends, rewards and slashes can be processed.

   - Jurors who voted are queued for rewards.

   - Jurors who didn't vote are queued for slashes.

   - Rewards and slashes are processed in batches during block initialization.

   - Escalated sessions only process slashes (no rewards).

9. **Exclusion Mechanism**:

   - Before deliberation starts, jurors can be excluded from duty.

   - Exclusion requires payment of an exclusion fee.

   - Excluded jurors are removed from selection and the summon count is decremented.

## Compromises and Improvements

- **Randomness Quality**: The pallet uses `insecure_randomness_collective_flip`, which is not cryptographically secure. A future improvement could use VRF (Verifiable Random Function) for true randomness.

- **No Juror Reputation**: The system doesn't track juror performance or reputation. Adding a reputation system could improve jury quality over time.

- **Batch Processing Limits**: Rewards and slashes are processed in fixed-size batches, which could delay distribution for large juror pools. Dynamic batch sizing could improve efficiency.

- **No Dispute Categories**: All disputes are treated equally. Adding dispute categories with different parameters (juror counts, voting periods) could improve flexibility.

- **Tied Vote Escalation**: Tied votes always escalate, even if the margin is small. A threshold-based escalation (e.g., escalate if within 5% of tie) could reduce unnecessary escalations.

- **No Appeal Mechanism**: Once a decision is made, there's no appeal process. Adding an appeal mechanism could improve fairness for edge cases.

- **Limited Context**: Context is stored as bounded bytes without structure. Adding structured context (categories, severity, etc.) could improve dispute clarity.

- **No Juror Rotation**: The same jurors can be selected repeatedly. Adding rotation mechanisms could prevent juror fatigue and improve diversity.

## Running the Project

### Prerequisites

- Rust & Cargo installed

- Substrate development environment set up

- frame omni-node

- Identity Registry pallet configured (for DID validation)

- Content Registry pallet configured (for ContentId validation)

- Insecure Randomness Collective Flip pallet configured

### Build and Test

1. Clone the repository:

   ```sh
   git clone <repository-url>
   cd era/pallets/pallet-context-court
   ```

2. Build the pallet:

   ```sh
   cargo build --release
   ```

3. Run tests:

   ```sh
   cargo test -p pallet-context-court
   ```

## Security Considerations

- **Economic Security**: Jurors must stake tokens, enabling slashing for misbehavior and creating economic commitment to honest participation.

- **Random Selection**: Juror selection uses on-chain randomness, preventing predictable jury composition and reducing collusion risk.

- **Double Voting Prevention**: The system prevents jurors from voting multiple times in the same session.

- **Temporal Constraints**: Disputes expire, preventing indefinite storage and ensuring timely resolution.

- **Permission Validation**: Only accounts with `Dispute` rights can create disputes or register as jurors, preventing unauthorized access.

- **Batch Processing Security**: Rewards and slashes are processed in controlled batches during block initialization, preventing DoS attacks through excessive processing.

- **Exclusion Fee**: The exclusion fee prevents frivolous exclusions and creates economic commitment to jury duty.

- **No Reward for Escalation**: Escalated sessions don't provide rewards, creating an incentive to resolve disputes efficiently in initial sessions.

- **Idempotent Operations**: Result calculation and reward distribution are idempotent, preventing duplicate processing.

## Decision Making Process

- **Storage Design**: The pallet uses multiple storage structures:
  - `Jurors`: List of all registered juror DIDs
  - `Dispute`: Map from ContentId to CourtSession
  - `EscalatedSession`: Map from ContentId to Escalated session data
  - `JurySelection`: Map from ContentId to selected jurors (temporary)
  - `JuryDuty`: Double-map tracking juror assignments
  - `PendingRewards`/`PendingSlashes`: Queues for batch processing

- **Randomness Usage**: The pallet uses `insecure_randomness_collective_flip` for juror selection. While not cryptographically secure, it provides sufficient randomness for this use case and can be upgraded to VRF in the future.

- **Batch Processing**: Rewards and slashes are processed in batches during `on_initialize` to manage gas costs. The batch size is configurable via `BatchRewardSize`.

- **Vote Counting**: The system uses simple majority voting. Tied votes trigger escalation to ensure a clear decision.

- **Escalation Logic**: Escalation occurs when votes are exactly tied (50/50). This ensures that close but not tied votes still produce a decision.

- **State Machine**: Disputes follow a clear state machine: Created → Jurors Summoned → Deliberation Started → Votes Cast → Result Calculated → Rewards Distributed.

- **Try-Mutate Pattern**: The pallet uses `try_mutate` for atomic updates to bounded vectors, ensuring consistency.

## Migration Story

This pallet can serve as a migration target for existing dispute resolution systems:

- **From Centralized Arbitration**: Centralized arbitration systems can be migrated by registering existing arbitrators as jurors and converting cases to disputes.

- **From Off-Chain Voting**: Off-chain voting systems can be enhanced by moving to on-chain voting with economic incentives and slashing.

- **From Simple Majority**: Systems using simple majority voting can be enhanced with the escalation mechanism for tied votes.

- **From Stateless Disputes**: Stateless dispute systems can be migrated by registering disputes on-chain for permanent record-keeping and reward distribution.

## Bigger Picture in the Substrate Ecosystem

This context court pallet fits into Substrate's modular architecture by providing **decentralized dispute resolution infrastructure**. It aligns with Substrate's strengths:

- **Interoperability**: The pallet integrates with identity and content registries, enabling end-to-end content verification and dispute resolution.

- **Composable Design**: Can be combined with governance pallets for DAO dispute resolution, treasury systems for reward funding, or reputation systems for juror quality tracking.

- **Cross-Chain Capabilities**: With XCM (Cross-Consensus Messaging), disputes registered on this chain could be resolved by jurors across parachains, enabling **cross-chain dispute resolution**.

- **DePIN Integration**: As part of the Era protocol, this pallet enables human verification of content context, bridging the gap between cryptographic content proofs and real-world meaning.

- **DAO Governance**: The jury system can be adapted for DAO governance, where disputes represent proposals and jurors represent voters.

- **Content Moderation**: The pallet provides infrastructure for decentralized content moderation, where communities can resolve disputes about content appropriateness.

- **Oracle Network**: The pallet can serve as a human oracle network, where jurors provide off-chain information (like content context) to on-chain systems.

This context court pallet provides a secure and efficient way to manage decentralized dispute resolution in frame-based networks. While the initial implementation focuses on content context disputes, future improvements can enhance its utility, security, and cross-chain capabilities within the Polkadot ecosystem.