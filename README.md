# 🎟️ StellarTix

> On-chain event ticketing with built-in anti-fraud, anti-scalping, and instant ownership transfer — powered by Stellar Soroban.

---

## Project Description

StellarTix is a decentralized event ticketing platform built on the Stellar that turns tickets into secure, verifiable digital assets.

Each ticket is:

Issued on-chain and bound to a user’s wallet
Impossible to duplicate or forge
Instantly verifiable at venue entry
Single-use to prevent fraud and reuse

It also supports smart contract–enforced resale rules, allowing organizers to prevent scalping and ensure fair pricing.

Powered by Stellar’s fast and low-cost transactions, StellarTix enables real-time ticket validation for concerts, sports, and large-scale events.

## Project Vision

StellarTix envisions a future where all event tickets are transparent, fraud-proof, and fully user-owned by default.

Built on the Stellar, it aims to:

Eliminate counterfeit tickets completely
End unfair scalping through programmable pricing rules
Give fans true ownership of their tickets
Enable instant, secure event entry worldwide

Beyond ticketing, StellarTix evolves into a full event ecosystem layer, powering loyalty rewards, fan engagement, and programmable digital event assets—creating a fair and trusted global event economy.

## 🧩 The Problem

A fan in Manila buys a concert ticket on a third-party resale site only to arrive at the venue and discover the ticket is counterfeit — or has already been used by someone else. Current ticketing systems rely on centralized databases that are opaque, easily spoofed, and routinely abused by scalpers who mark up prices 3–5×.

**Who suffers:** Event-goers in SEA (Philippines, Indonesia, Vietnam) who have no way to verify ticket authenticity before paying.

**Cost of friction:** Lost money, denied entry, and zero recourse once a fraudulent ticket is purchased.

---

## ✅ The Solution

StellarTix issues every ticket as a unique, immutable record on Stellar via a Soroban smart contract. Each ticket is:

- **Cryptographically bound** to its buyer's wallet address at the moment of purchase
- **Tamper-evident** via an integrity hash stored on-chain
- **Anti-scalping** enforced by a hard resale price cap set by the organizer
- **Gate-validated** through a single on-chain scan that marks the ticket as used, making re-entry impossible

Stellar is essential here because of its sub-second finality, near-zero fees (< $0.001 per transaction), and built-in XLM payment primitives — making real-time gate scanning and micropayment resale economically viable.

---

## 🌟 Stellar Features Used

| Feature | Usage |
|---|---|
| **Soroban Smart Contracts** | Core ticket registry, tamper detection, ownership transfer, resale price cap |
| **XLM Transfers** | Ticket purchase payments and resale transactions |
| **Custom Tokens** | Optional event-specific loyalty tokens for attendees |
| **Trustlines** | Required for holders of custom event asset tokens |
| **On-chain Events** | Audit trail for issuance, verification, scanning, and transfer |

---

## 🎯 Target Users

| Segment | Detail |
|---|---|
| **Event-goers / Fans** | Concertgoers, sports fans, festival attendees in SEA who distrust resale markets |
| **Event Organizers** | Concert promoters, sports teams, bootcamp operators who want fraud-proof ticketing |
| **Gate Staff** | Venue security using a mobile scanner app that calls `verify_ticket` and `scan_ticket` |

---

## 🏗️ Core MVP Flow

```
Organizer deploys contract → calls initialize()
         │
         ▼
Organizer calls issue_ticket(ticket_id, buyer_wallet, event, seat, price, cap, hash)
         │                         [Ticket stored on-chain, event emitted]
         ▼
Fan receives ticket_id + QR code linking to their wallet
         │
         ▼
At venue: Gate scanner calls verify_ticket(ticket_id, expected_hash)
         │                         [Returns true/false + emits verify event]
         ▼
         └── true  → call scan_ticket() → ticket.is_used = true → Fan enters ✅
         └── false → Entry denied, alert raised ❌
```

**Demo runtime: < 90 seconds** — issue one ticket, verify it, scan it, then attempt re-entry (rejected).

---

## 🗓️ Suggested MVP Timeline

| Week | Milestone |
|---|---|
| Week 1 | Soroban contract complete, all 3 tests passing, deployed to testnet |
| Week 2 | React web app: organizer dashboard (issue tickets), fan wallet page |
| Week 3 | Gate scanner mobile view (verify + scan), resale listing UI |
| Week 4 | End-to-end demo polish, pitch deck, testnet walkthrough video |

---

## 🏆 Why This Wins

StellarTix directly solves a real, painful problem for millions of event-goers in Southeast Asia with a demo-able, sub-2-minute flow. It uses Soroban's composability for tamper detection and price enforcement — features impossible to replicate with a simple payment rail — making it a strong fit for Stellar's hackathon criteria of real-world adoption and local economic impact.

---

## 🔧 Prerequisites

- **Rust toolchain** — Install via [rustup.rs](https://rustup.rs)
  ```bash
  rustup target add wasm32-unknown-unknown
  ```
- **Stellar CLI** — v22.0.0 or later
  ```bash
  cargo install --locked stellar-cli --features opt
  ```
- **Node.js** ≥ 18 (for frontend, optional for contract work)

---

## 🔨 Build

```bash
# Build the Wasm contract (output: target/wasm32-unknown-unknown/release/stellartix.wasm)
stellar contract build
```

---

## 🧪 Test

```bash
# Run all 3 contract tests
cargo test

# Run with log output visible
RUST_LOG=debug cargo test -- --nocapture
```

Expected output:
```
test tests::test_happy_path_issue_and_verify ... ok
test tests::test_duplicate_ticket_rejected ... ok
test tests::test_state_reflects_correct_data_after_issuance ... ok

test result: ok. 3 passed; 0 failed
```

---

## 🚀 Deploy to Testnet

```bash
# 1. Generate a testnet keypair and fund it via Friendbot
stellar keys generate --global alice --network testnet
stellar keys fund alice --network testnet

# 2. Deploy the compiled Wasm
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/stellartix.wasm \
  --source alice \
  --network testnet

# Returns: CONTRACT_ID (save this — you'll need it for all invocations)
```

---

## 🖥️ Sample CLI Invocations

Replace `<CONTRACT_ID>` with your deployed contract address and `<ADMIN_ADDRESS>` / `<BUYER_ADDRESS>` with Stellar account addresses.

### Initialize the contract
```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source alice \
  --network testnet \
  -- initialize \
  --admin <ADMIN_ADDRESS>
```

### Issue a ticket
```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source alice \
  --network testnet \
  -- issue_ticket \
  --ticket_id "EVT2025-0001" \
  --owner <BUYER_ADDRESS> \
  --event_name "Eraserheads Reunion Concert" \
  --event_date 1780000000 \
  --seat "FLOOR-A12" \
  --issue_price 500000000 \
  --max_resale_price 750000000 \
  --integrity_hash "abc123hash"
```

### Verify a ticket (gate scanner)
```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source alice \
  --network testnet \
  -- verify_ticket \
  --ticket_id "EVT2025-0001" \
  --expected_hash "abc123hash"
# Returns: true
```

### Scan a ticket at entry (marks as used)
```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source alice \
  --network testnet \
  -- scan_ticket \
  --ticket_id "EVT2025-0001"
```

### List a ticket for resale
```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source <BUYER_ADDRESS> \
  --network testnet \
  -- list_for_resale \
  --ticket_id "EVT2025-0001" \
  --asking_price 700000000
```

### Transfer ticket to new buyer
```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source <NEW_BUYER_ADDRESS> \
  --network testnet \
  -- transfer_ticket \
  --ticket_id "EVT2025-0001" \
  --new_owner <NEW_BUYER_ADDRESS> \
  --integrity_hash "newownerHash456"
```

### Read ticket data
```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source alice \
  --network testnet \
  -- get_ticket \
  --ticket_id "EVT2025-0001"
```

---

## 🔮 Optional Enhancements (Bonus Points)

- **AI Fraud Detection** — Feed on-chain transfer history to an AI model that flags suspicious bulk-buying patterns before scalpers can re-list
- **Offline QR Validation** — Pre-sign ticket proofs so gate scanners work without network connectivity (Stellar SEP support)
- **Anchor Integration** — Allow ticket purchase via GCash / Maya (Philippines e-wallets) through a local Stellar anchor, removing the need for fans to hold XLM directly
- **Royalty on Resale** — Smart contract automatically routes a % of every resale back to the artist/organizer wallet

---

## 📄 License

MIT License

Copyright (c) 2025 StellarTix Contributors

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

---
## Deployed Contract Link
[1] https://stellar.expert/explorer/testnet/tx/9062f2a65de6a0fd200cbc635f4050da49d07a6a4d1204da0c2cf27a7f13578a
[2] https://lab.stellar.org/r/testnet/contract/CBO4HL5TXD3O5LYNMTA5MZUGD7VFJMHJR7DYP4JTAVDT6VSGWJXJTKDV


## Future Scope
The development is still in progress

