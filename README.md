# 🎟️ Event Ticket

> Tamper-proof, on-chain event ticketing on Stellar Soroban — kill scalping, kill fakes, reward real fans.

---

## Project Description
Event Ticket is a decentralized ticketing system built on the Stellar using Soroban smart contracts that replaces vulnerable centralized ticketing platforms with a transparent, tamper-proof on-chain system.

Each ticket is represented as a unique cryptographic record (SHA-256 hash + wallet address) stored on-chain. This ensures every ticket is authentic, traceable, and impossible to duplicate or reuse. At event entry, a gate scanner performs a real-time on-chain verification, instantly confirming validity before allowing access.

The system also introduces anti-scalping enforcement, allowing organizers to track and control ticket transfers while optionally capping resale activity. Buyers who purchase through official channels can be rewarded with small XLM incentives, encouraging fair distribution and reducing reliance on secondary markets.

By leveraging Stellar’s fast finality and low transaction fees, Event Ticket enables scalable, real-time verification suitable for concerts, festivals, and high-volume live events.

---

## Project Vision
Event Ticket envisions a future where every event ticket is verifiable, fraud-resistant, and fairly distributed by default.

Built on the Stellar, the platform aims to eliminate counterfeit tickets, dismantle exploitative scalping systems, and restore trust between fans and event organizers.

Its long-term vision is to create a fair global event economy, where:

- Every ticket is provably authentic before purchase
- Scalpers are unable to manipulate resale markets
- Fans have true ownership and control over their tickets
- Organizers maintain full transparency over ticket distribution

Beyond ticketing, Event Ticket evolves into a broader on-chain event infrastructure layer, enabling loyalty rewards, fan engagement systems, and programmable event assets that strengthen the connection between creators and audiences while ensuring fairness and transparency at scale.

 ---

## Problem

A Filipino concert-goer pays ₱4,000 for a ticket on a resale site only to discover it's counterfeit at the gate. No refund, no entry. Event organisers lose revenue to scalpers who buy in bulk and resell at 3× face value. Existing solutions rely on centralised databases that are opaque, hackable, and slow to verify.

## Solution

Event Ticket registers every ticket as a unique on-chain record (SHA-256 hash + owner wallet) via a Soroban smart contract. The gate scanner hashes the QR code and calls `verify_ticket` — an instant on-chain check that proves authenticity and prevents double-entry. Buyers earn an XLM micro-reward for purchasing through the official dApp. Transfers are tracked on-chain so organisers can cap resale margins.

---

## Stellar Features Used

| Feature | Purpose |
|---|---|
| **Soroban smart contracts** | Core ticket registry, duplicate detection, tamper verification |
| **XLM transfers** | Buyer rewards paid on first verified purchase |
| **Custom tokens** (optional) | Organiser-issued event-specific NFT-style tickets |
| **Trustlines** | Gating access to premium event token tiers |
| **On-chain events** | `REGISTER`, `VERIFY`, `TRANSFER`, `REWARD` — indexable off-chain |

---

## Public Contract Functions

| Function | Who calls it | What it does |
|---|---|---|
| `init(admin)` | Deployer | Stores admin address once |
| `register_ticket(hash, owner)` | Admin/organiser | Issues ticket, rejects duplicates |
| `verify_ticket(hash, expected_hash)` | Gate scanner | Returns bool + emits VERIFY event |
| `use_ticket(hash)` | Admin/gate | Marks ticket consumed |
| `reward_buyer(hash, amount)` | Admin backend | Logs XLM reward on-chain |
| `transfer_ticket(hash, new_owner)` | Current owner | Peer-to-peer resale with on-chain audit trail |
| `get_ticket(hash)` | Anyone | Read the full TicketRecord |
| `get_owner_tickets(wallet)` | Anyone | List all tickets owned by a wallet |

---

## Suggested MVP Timeline

| Day | Milestone |
|---|---|
| 1–2 | Soroban contract written and unit-tested |
| 3 | Deploy to Stellar Testnet; wire up CLI invocations |
| 4–5 | React dApp: organiser dashboard (issue), buyer portal (wallet connect, verify), gate scanner page |
| 6 | XLM reward flow end-to-end test |
| 7 | Demo rehearsal & README polish |

---

## Prerequisites

```bash
# Rust toolchain with wasm32 target
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup target add wasm32-unknown-unknown

# Soroban CLI (v22 or later)
cargo install --locked stellar-cli --features opt

# Verify
stellar --version   # should print stellar 22.x.x
```

---

## Build

```bash
# From the project root
stellar contract build
# Output: target/wasm32-unknown-unknown/release/event_ticket.wasm
```

---

## Test

```bash
cargo test
# Runs all 3 unit tests (happy path, duplicate rejection, state verification)
```

---

## Deploy to Testnet

```bash
# 1. Fund a testnet identity
stellar keys generate deployer --network testnet
stellar keys fund deployer --network testnet

# 2. Deploy the compiled Wasm
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/event_ticket.wasm \
  --source deployer \
  --network testnet

# Returns a CONTRACT_ID — save it as $CONTRACT_ID

# 3. Initialise with admin wallet
stellar contract invoke \
  --id $CONTRACT_ID \
  --source deployer \
  --network testnet \
  -- init \
  --admin $(stellar keys address deployer)
```

---

## Sample CLI Invocations

### Register a Ticket

```bash
stellar contract invoke \
  --id $CONTRACT_ID \
  --source deployer \
  --network testnet \
  -- register_ticket \
  --ticket_hash abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890 \
  --owner GBUYER123EXAMPLEADDRESS
```

### Verify a Ticket (Gate Scanner)

```bash
stellar contract invoke \
  --id $CONTRACT_ID \
  --source deployer \
  --network testnet \
  -- verify_ticket \
  --ticket_hash abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890 \
  --expected_hash abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890
# Returns: true (valid) or false (tampered/used)
```

### Log a Buyer Reward

```bash
stellar contract invoke \
  --id $CONTRACT_ID \
  --source deployer \
  --network testnet \
  -- reward_buyer \
  --ticket_hash abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890 \
  --amount_stroops 50000000
# 50_000_000 stroops = 5 XLM
```

### Transfer Ticket (Resale)

```bash
stellar contract invoke \
  --id $CONTRACT_ID \
  --source GBUYER123EXAMPLEADDRESS \
  --network testnet \
  -- transfer_ticket \
  --ticket_hash abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890 \
  --new_owner GNEWBUYER456EXAMPLEADDRESS
```

---

## Repository Structure

```
event_ticket/
├── Cargo.toml
├── README.md
└── src/
    ├── lib.rs      ← Soroban contract
    └── test.rs     ← 3 unit tests
```

---

## Why This Wins

Stellar's 5-second finality and near-zero fees make real-time gate verification practical at scale. On-chain event sourcing (`REGISTER`, `VERIFY`, `TRANSFER`, `REWARD`) gives organisers a tamper-proof audit trail that centralised ticketing giants cannot match. The XLM reward mechanic directly incentivises official-channel purchases, attacking the root cause of scalping in Southeast Asian live events markets.

---

## License

MIT © 2026 Event Ticket

## Deployed Contract Link
[1] https://stellar.expert/explorer/testnet/tx/9062f2a65de6a0fd200cbc635f4050da49d07a6a4d1204da0c2cf27a7f13578a
[2] https://lab.stellar.org/r/testnet/contract/CBO4HL5TXD3O5LYNMTA5MZUGD7VFJMHJR7DYP4JTAVDT6VSGWJXJTKDV


## Future Scope
The development is still in progress

