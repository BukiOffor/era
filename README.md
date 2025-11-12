# Era: The Ledger of Reality

**Era** is a Substrate-based blockchain protocol designed to create an immutable, verifiable ledger of digital content authenticity. In an age of AI-generated content and deepfakes, Era provides cryptographic proof that content is real, unaltered, and created at a specific time by a verified device.

## Project Overview

Era addresses the fundamental problem of digital trust by flipping the traditional approach: instead of trying to detect what's fake, Era proves what's real. The protocol creates an unforgeable link between digital content and physical devices using hardware security modules (Secure Enclave on iOS, TEE on Android), enabling verifiable content authenticity at the moment of creation.

The Era blockchain is built as an **AppChain** optimized for high-throughput, low-cost ingestion of content proofs. It consists of three core pallets that work together to provide end-to-end content verification:

1. **Identity Registry** - Manages decentralized identities (DIDs) and permissions
2. **Content Registry** - Stores cryptographic proofs of content authenticity
3. **Context Court** - Provides decentralized dispute resolution for content context

## Architecture

### Core Components

#### 1. Identity Registry Pallet
The foundation of the Era protocol, managing Decentralized Identifiers (DIDs) and their associated permissions. This pallet enables:
- DID creation and management
- Granular rights system (Update, Impersonate, Dispute)
- Device registration and verification
- Temporal permissions (permanent and time-bound)

**📖 [Read the Identity Registry documentation →](./pallets/pallet-identity-registry/README.md)**

#### 2. Content Registry Pallet
The heart of content verification, storing immutable proofs of content authenticity. This pallet provides:
- Cryptographic content registration (Blake2-256 hashing)
- Content-to-DID binding
- Device-based origin verification
- Immutable content ledger

**📖 [Read the Content Registry documentation →](./pallets/pallet-content-registry/README.md)**

#### 3. Context Court Pallet
A decentralized dispute resolution system that bridges the gap between cryptographic content verification and contextual truth. This pallet enables:
- Jury-based dispute resolution
- Two-tier voting system (initial jury + escalation)
- Economic incentives (rewards for participation, slashes for misbehavior)
- Batch processing for efficient reward distribution

**📖 [Read the Context Court documentation →](./pallets/pallet-context-court/README.md)**

### How They Work Together

```
┌─────────────────────┐
│  Identity Registry  │  ← Creates DIDs and manages permissions
└──────────┬──────────┘
           │
           │ (validates permissions)
           ▼
┌─────────────────────┐
│  Content Registry   │  ← Registers content with cryptographic proofs
└──────────┬──────────┘
           │
           │ (disputes about context)
           ▼
┌─────────────────────┐
│   Context Court     │  ← Resolves disputes through jury voting
└─────────────────────┘
```

1. **Identity Creation**: Users create DIDs in the Identity Registry and register devices
2. **Content Registration**: Users with appropriate permissions register content in the Content Registry, creating immutable proofs
3. **Dispute Resolution**: When content context is disputed, the Context Court enables decentralized resolution through juror voting

## Key Features

- **Hardware-Based Security**: Leverages Secure Enclave (iOS) and TEE (Android) for device-level signing
- **Cryptographic Integrity**: Content IDs derived from content hashes ensure tamper detection
- **Decentralized Identity**: Self-sovereign DIDs with granular permission management
- **Economic Security**: Staking and slashing mechanisms ensure honest participation
- **Immutable Ledger**: Once registered, content proofs cannot be modified or deleted
- **Composable Design**: Modular pallets that integrate seamlessly with other Substrate pallets

## The Problem Era Solves

In the current digital landscape:
- AI-generated content is indistinguishable from real content
- Deepfakes can fool even experts
- Context can be manipulated (real photos with false claims)
- There's no reliable way to prove digital content authenticity

Era solves this by:
- Creating cryptographic proofs at the moment of content creation
- Binding content to physical devices through hardware security
- Providing an immutable, on-chain record of authenticity
- Enabling human verification of content context through decentralized dispute resolution

## Getting Started

### Prerequisites

- Rust & Cargo installed ([rustup.rs](https://rustup.rs/))
- Substrate development environment
- frame omni-node (for local development)

### Building the Project

1. Clone the repository:

   ```sh
   git clone <repository-url>
   cd era
   ```

2. Build the project:

   ```sh
   cargo build --release
   ```

3. Run tests:

   ```sh
   cargo test
   ```

### Running a Local Node

```sh
cargo run --release -- --dev
```

## Project Structure

```
era/
├── node/                    # Substrate node implementation
├── runtime/                # Runtime configuration
├── pallets/
│   ├── pallet-identity-registry/    # DID and identity management
│   ├── pallet-content-registry/     # Content proof storage
│   ├── pallet-context-court/        # Dispute resolution system
│   └── shared/                      # Shared traits and types
├── Story.md                 # The story and vision behind Era
└── README.md               # This file
```

## Documentation

- **[Identity Registry Pallet](./pallets/pallet-identity-registry/README.md)** - Complete documentation for the identity management system
- **[Content Registry Pallet](./pallets/pallet-content-registry/README.md)** - Complete documentation for content verification
- **[Context Court Pallet](./pallets/pallet-context-court/README.md)** - Complete documentation for dispute resolution
- **[Story.md](./Story.md)** - The inspiration, challenges, and vision behind Era

## Use Cases

- **Journalism**: Verify the authenticity of news photos and videos
- **Legal Documentation**: Create immutable records of evidence
- **Scientific Research**: Verify the authenticity of research data
- **Social Media**: Enable platforms to verify user-generated content
- **Insurance**: Verify claims with authentic documentation
- **Content Marketplaces**: Prove authenticity of digital assets

## Security Considerations

- **Hardware Security**: Content proofs are bound to hardware security modules
- **Cryptographic Hashing**: Blake2-256 ensures tamper detection
- **Economic Incentives**: Staking and slashing align incentives for honest behavior
- **Permission System**: Granular rights prevent unauthorized operations
- **Immutable Records**: Once registered, proofs cannot be modified

## Roadmap

- [ ] Mobile app with Secure Enclave/TEE integration
- [ ] IPFS integration for content storage
- [ ] API and SDK for third-party integration
- [ ] Cross-chain verification via XCM
- [ ] Reputation system for jurors
- [ ] Firmware-level integration with device manufacturers

## Contributing

We welcome contributions! Please see our contributing guidelines (to be added) for details on:
- Code style and standards
- Testing requirements
- Pull request process
- Issue reporting

## License

This project is licensed under the terms specified in the [LICENSE](./LICENSE) file.

## Bigger Picture

Era is more than a blockchain—it's infrastructure for a more truthful digital age. By creating a permanent, immutable, and decentralized "ledger of reality," Era enables:

- **Trust in Digital Media**: Know that what you see is real
- **Accountability**: Trace content back to its source
- **Verification**: Prove authenticity without relying on central authorities
- **Decentralized Truth**: Human judgment for contextual verification

## Learn More

- Read [Story.md](./Story.md) to understand the inspiration and vision behind Era
- Explore individual pallet documentation for technical details
- Check out the [Substrate documentation](https://docs.substrate.io/) to understand the underlying framework

---

**Era: If it matters, it's on Era.**