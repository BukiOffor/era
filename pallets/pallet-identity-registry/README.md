# Identity Registry Pallet for Substrate

## Project Overview

This project implements an **Identity Registry Pallet** for Substrate, designed to facilitate decentralized identity (DID) management on-chain. The pallet provides a comprehensive system for creating and managing Decentralized Identifiers (DIDs), managing signatories with granular rights, and registering devices associated with identities. The pallet serves as the foundational identity layer for the Era protocol, enabling secure, permissioned operations across the ecosystem. The design prioritizes flexibility in access control through a rights-based system that supports both permanent and temporary permissions, ensuring that identity management remains both secure and adaptable to various use cases.

## Background and Considerations

Decentralized identity management is essential for building trustless systems where users need to prove ownership, grant permissions, and manage access across different services. In designing this pallet, key considerations included:

- **Decentralization**: DIDs are self-sovereign and do not rely on centralized authorities for creation or management.

- **Flexibility**: The rights system allows for fine-grained permissions (Update, Impersonate, Dispute) with both permanent and temporary durations.

- **Device Management**: Support for multiple devices per DID enables users to manage their identity across different hardware.

- **Security**: A deposit mechanism prevents spam DID creation and ensures commitment to the network.

- **Composability**: The pallet implements the `DidManager` trait, allowing other pallets to query identity information and validate permissions.

- **Temporal Rights**: Support for time-bound permissions enables temporary access grants and automatic expiration.

## State Transition Function Design

The core logic of the pallet is structured as follows:

1. **DID Creation**:

   - A new DID is created with an initial set of signatories.

   - The creator automatically receives the `Update` right permanently.

   - A deposit is held from the creator's account to prevent spam.

   - The DID is stored with its associated signatories.

2. **Rights Management**:

   - Signatories with `Update` rights can grant rights to other accounts.

   - Rights can be permanent or temporary (with block-based expiration).

   - Rights can be removed by authorized signatories.

   - The system supports three base rights: `Update`, `Impersonate`, and `Dispute`.

3. **Device Registration**:

   - Signatories with `Update` rights can register devices for a DID.

   - Multiple devices can be associated with a single DID.

   - Devices can be removed when no longer needed.

4. **Permission Validation**:

   - The pallet provides a `is_valid_signatory` function that checks if an account has a specific right for a DID.

   - Temporary rights are automatically validated against the current block number.

   - The pallet implements the `DidManager` trait for cross-pallet integration.

## Compromises and Improvements

- **No DID Deletion**: Once created, DIDs cannot be deleted, which could lead to storage bloat over time. A future improvement could add a deletion mechanism with appropriate safeguards.

- **No Right Delegation Chain**: Rights cannot be delegated further by recipients. This prevents complex delegation chains but also limits flexibility.

- **Limited Right Types**: Currently supports three base rights. The system could be extended to support custom right types for domain-specific use cases.

- **No Reputation System**: The pallet does not track reputation or history of DID operations, which could be valuable for trust scoring.

- **Device Metadata**: Devices are stored as opaque types without metadata. Adding device metadata (model, registration date, etc.) could enhance security and usability.

- **No Multi-Sig Support**: While multiple signatories are supported, there's no threshold-based multi-signature mechanism for critical operations.

## Running the Project

### Prerequisites

- Rust & Cargo installed

- Substrate development environment set up

- frame omni-node

### Build and Test

1. Clone the repository:

   ```sh
   git clone <repository-url>
   cd era/pallets/pallet-identity-registry
   ```

2. Build the pallet:

   ```sh
   cargo build --release
   ```

3. Run tests:

   ```sh
   cargo test -p pallet-identity-registry
   ```

## Security Considerations

- **Deposit Mechanism**: The hold amount required for DID creation prevents spam and ensures economic commitment to the network.

- **Right Validation**: All operations check for appropriate rights before execution, preventing unauthorized access.

- **Temporal Rights**: Temporary rights automatically expire based on block numbers, preventing stale permissions from persisting indefinitely.

- **Device Ownership**: Device registration requires `Update` rights, ensuring only authorized signatories can manage devices.

- **Immutable Signatories**: The initial signatory list is set at creation and cannot be modified directly. Rights must be granted to new signatories through the rights system.

- **No Right Escalation**: The system prevents privilege escalation by requiring `Update` rights to grant rights, creating a clear permission hierarchy.

## Decision Making Process

- **Storage Design**: The pallet uses separate storage maps for signatories, rights, and devices to optimize query performance and reduce storage costs.

- **BoundedVec Usage**: All collections use `BoundedVec` to prevent unbounded storage growth and enable accurate weight calculation.

- **Trait Implementation**: The `DidManager` trait implementation allows other pallets (like content-registry and context-court) to query identity information without tight coupling.

- **Right Duration Enum**: Using an enum for right duration (Permanent vs Temporary) provides type safety and clear semantics compared to using optional block numbers.

- **Event Emission**: All state-changing operations emit events, enabling off-chain systems to track identity changes and build indexes.

## Migration Story

This pallet can serve as a migration target for existing identity systems:

- **From Centralized Identity**: Organizations can migrate from centralized identity providers by creating DIDs for existing users and granting appropriate rights.

- **From Stateless Identity**: Stateless identity systems (like those using only cryptographic signatures) can be enhanced by registering DIDs on-chain for discoverability and rights management.

- **From Other DID Systems**: DIDs from other systems (like W3C DIDs) can be registered in this pallet, with the on-chain registry serving as a verifiable credential store.

## Bigger Picture in the Substrate Ecosystem

This identity registry pallet fits into Substrate's modular architecture by providing **foundational identity infrastructure**. It aligns with Substrate's strengths:

- **Interoperability**: The `DidManager` trait enables seamless integration with other pallets, allowing content-registry and context-court to validate permissions without direct dependencies.

- **Composable Design**: Can be combined with governance pallets, treasury systems, or NFT pallets to create sophisticated identity-based applications.

- **Cross-Chain Capabilities**: With XCM (Cross-Consensus Messaging), DIDs registered on this chain could be referenced and validated across parachains, enabling **cross-chain identity**.

- **DePIN Integration**: As part of the Era protocol, this pallet enables device-based identity verification, linking physical devices to on-chain identities for content authenticity.

- **DAO Governance**: The rights system can be extended to support DAO governance, where DIDs represent organizations and rights represent voting or proposal powers.

This identity registry pallet provides a secure and flexible foundation for decentralized identity management in frame-based networks. While the initial implementation focuses on core identity operations, future improvements can enhance its utility, security, and cross-chain capabilities within the Polkadot ecosystem.