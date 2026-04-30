//! StellarTix — On-chain Event Ticketing & Anti-Fraud Contract
//!
//! This Soroban smart contract manages the full lifecycle of an event ticket:
//! issuance, transfer, validation at the gate, and resale price enforcement.
//! Each ticket is a unique on-chain record bound to an owner's wallet address.
//! Duplicate issuance is blocked and tampered tickets are detected on-chain.

#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short,
    Address, Env, String, Symbol, Vec, log,
};

// ─────────────────────────────────────────────
// Storage key namespaces
// ─────────────────────────────────────────────

/// Top-level storage key enum used as keys in the contract's persistent map.
#[contracttype]
pub enum DataKey {
    /// Stores TicketInfo for a given ticket_id (String)
    Ticket(String),
    /// Stores the admin address (organizer who deployed the contract)
    Admin,
    /// Counter tracking total tickets issued for an event
    TicketCount,
}

// ─────────────────────────────────────────────
// Data structures
// ─────────────────────────────────────────────

/// Full metadata for a single ticket stored on-chain.
#[contracttype]
#[derive(Clone)]
pub struct TicketInfo {
    /// Unique ticket ID (e.g. "EVT2025-0042")
    pub ticket_id: String,
    /// Wallet address of the current ticket holder
    pub owner: Address,
    /// Name of the event (e.g. "Eraserheads Reunion Concert")
    pub event_name: String,
    /// Unix timestamp of the event date
    pub event_date: u64,
    /// Seat or section label (e.g. "FLOOR-A12")
    pub seat: String,
    /// Original issue price in stroops (1 XLM = 10,000,000 stroops)
    pub issue_price: i128,
    /// Maximum allowed resale price in stroops (caps scalping)
    pub max_resale_price: i128,
    /// Whether the ticket has been scanned / used at the gate
    pub is_used: bool,
    /// Whether the ticket is currently listed for resale
    pub for_sale: bool,
    /// Current asking price if listed for resale (in stroops)
    pub resale_price: i128,
    /// SHA-256 hash of (ticket_id + owner + event_name) for tamper detection
    pub integrity_hash: String,
}

// ─────────────────────────────────────────────
// Error codes emitted via panic strings
// ─────────────────────────────────────────────
// Soroban panics are surfaced to callers as error messages.
// We use named string constants for readability.

const ERR_UNAUTHORIZED: &str      = "unauthorized";
const ERR_TICKET_EXISTS: &str     = "ticket_already_exists";
const ERR_TICKET_NOT_FOUND: &str  = "ticket_not_found";
const ERR_TICKET_USED: &str       = "ticket_already_used";
const ERR_NOT_OWNER: &str         = "caller_is_not_owner";
const ERR_NOT_FOR_SALE: &str      = "ticket_not_listed_for_sale";
const ERR_PRICE_TOO_HIGH: &str    = "resale_price_exceeds_cap";
const ERR_INTEGRITY_FAIL: &str    = "integrity_hash_mismatch";

// ─────────────────────────────────────────────
// Contract
// ─────────────────────────────────────────────

#[contract]
pub struct StellarTixContract;

#[contractimpl]
impl StellarTixContract {

    // ──────────────────────────────────────────
    // INIT: Set admin (event organizer)
    // Must be called once after deployment.
    // ──────────────────────────────────────────

    /// Initializes the contract by storing the deployer as the admin.
    /// Only the admin can issue tickets.
    pub fn initialize(env: Env, admin: Address) {
        // Require admin signature so only they can set themselves
        admin.require_auth();
        env.storage().persistent().set(&DataKey::Admin, &admin);
        env.storage().persistent().set(&DataKey::TicketCount, &0u32);
    }

    // ──────────────────────────────────────────
    // ISSUE_TICKET: Organizer mints a new ticket
    // ──────────────────────────────────────────

    /// Issues a new ticket on-chain. Only callable by the admin.
    /// Prevents duplicate ticket IDs and stores full metadata.
    ///
    /// # Arguments
    /// * `ticket_id`       — Unique identifier for this ticket
    /// * `owner`           — Wallet address of the buyer
    /// * `event_name`      — Human-readable event name
    /// * `event_date`      — Unix timestamp of the event
    /// * `seat`            — Seat / section label
    /// * `issue_price`     — Price paid in stroops
    /// * `max_resale_price`— Hard cap for secondary market pricing
    /// * `integrity_hash`  — Hash of key fields for tamper detection
    pub fn issue_ticket(
        env: Env,
        ticket_id: String,
        owner: Address,
        event_name: String,
        event_date: u64,
        seat: String,
        issue_price: i128,
        max_resale_price: i128,
        integrity_hash: String,
    ) {
        // Only admin (event organizer) may issue tickets
        let admin: Address = env.storage().persistent()
            .get(&DataKey::Admin)
            .unwrap_or_else(|| panic!("{}", ERR_UNAUTHORIZED));
        admin.require_auth();

        // Block duplicate issuance — each ticket_id must be globally unique
        let key = DataKey::Ticket(ticket_id.clone());
        if env.storage().persistent().has(&key) {
            panic!("{}", ERR_TICKET_EXISTS);
        }

        // Build and persist the ticket record
        let ticket = TicketInfo {
            ticket_id: ticket_id.clone(),
            owner: owner.clone(),
            event_name: event_name.clone(),
            event_date,
            seat,
            issue_price,
            max_resale_price,
            is_used: false,
            for_sale: false,
            resale_price: 0,
            integrity_hash,
        };

        env.storage().persistent().set(&key, &ticket);

        // Increment the total ticket counter for this contract
        let count: u32 = env.storage().persistent()
            .get(&DataKey::TicketCount)
            .unwrap_or(0);
        env.storage().persistent().set(&DataKey::TicketCount, &(count + 1));

        // Emit an event so indexers and frontends can react
        env.events().publish(
            (symbol_short!("issued"), ticket_id),
            owner,
        );

        log!(&env, "Ticket issued successfully");
    }

    // ──────────────────────────────────────────
    // VERIFY_TICKET: Gate scanner checks validity
    // ──────────────────────────────────────────

    /// Returns true if the ticket is valid (exists, not used, integrity intact).
    /// Also emits a verification event for audit trail purposes.
    /// Called by gate staff / validator app at entry.
    ///
    /// # Arguments
    /// * `ticket_id`       — Ticket to verify
    /// * `expected_hash`   — Hash recomputed off-chain from ticket fields
    pub fn verify_ticket(
        env: Env,
        ticket_id: String,
        expected_hash: String,
    ) -> bool {
        let key = DataKey::Ticket(ticket_id.clone());

        // Ticket must exist
        let ticket: TicketInfo = env.storage().persistent()
            .get(&key)
            .unwrap_or_else(|| panic!("{}", ERR_TICKET_NOT_FOUND));

        // Ticket must not have been scanned before
        if ticket.is_used {
            env.events().publish(
                (symbol_short!("verify"), ticket_id),
                false,
            );
            return false;
        }

        // Integrity check: compare stored hash with recomputed hash
        // A mismatch means the ticket data was tampered with off-chain
        let valid = ticket.integrity_hash == expected_hash;

        env.events().publish(
            (symbol_short!("verify"), ticket_id),
            valid,
        );

        valid
    }

    // ──────────────────────────────────────────
    // SCAN_TICKET: Mark ticket as used at entry
    // ──────────────────────────────────────────

    /// Marks a ticket as scanned/used so it cannot be re-used.
    /// Only the admin (gate system) can call this.
    pub fn scan_ticket(env: Env, ticket_id: String) {
        let admin: Address = env.storage().persistent()
            .get(&DataKey::Admin)
            .unwrap_or_else(|| panic!("{}", ERR_UNAUTHORIZED));
        admin.require_auth();

        let key = DataKey::Ticket(ticket_id.clone());
        let mut ticket: TicketInfo = env.storage().persistent()
            .get(&key)
            .unwrap_or_else(|| panic!("{}", ERR_TICKET_NOT_FOUND));

        if ticket.is_used {
            panic!("{}", ERR_TICKET_USED);
        }

        ticket.is_used = true;
        env.storage().persistent().set(&key, &ticket);

        env.events().publish(
            (symbol_short!("scanned"), ticket_id),
            ticket.owner,
        );
    }

    // ──────────────────────────────────────────
    // LIST_FOR_RESALE: Ticket owner lists on secondary market
    // ──────────────────────────────────────────

    /// Allows the current ticket owner to list their ticket for resale.
    /// Enforces the max_resale_price cap set by the organizer to prevent scalping.
    pub fn list_for_resale(
        env: Env,
        ticket_id: String,
        asking_price: i128,
    ) {
        let key = DataKey::Ticket(ticket_id.clone());
        let mut ticket: TicketInfo = env.storage().persistent()
            .get(&key)
            .unwrap_or_else(|| panic!("{}", ERR_TICKET_NOT_FOUND));

        // Only the current owner may list it
        ticket.owner.require_auth();

        if ticket.is_used {
            panic!("{}", ERR_TICKET_USED);
        }

        // Enforce anti-scalping price cap
        if asking_price > ticket.max_resale_price {
            panic!("{}", ERR_PRICE_TOO_HIGH);
        }

        ticket.for_sale = true;
        ticket.resale_price = asking_price;
        env.storage().persistent().set(&key, &ticket);

        env.events().publish(
            (symbol_short!("listed"), ticket_id),
            asking_price,
        );
    }

    // ──────────────────────────────────────────
    // TRANSFER_TICKET: Buyer purchases resale ticket
    // ──────────────────────────────────────────

    /// Transfers ownership of a listed ticket from seller to buyer.
    /// XLM payment is handled at the application layer (via Stellar payment op).
    /// This function updates on-chain ownership after payment confirmation.
    ///
    /// # Arguments
    /// * `ticket_id`      — Ticket being purchased
    /// * `new_owner`      — Buyer's wallet address
    /// * `integrity_hash` — New hash reflecting the ownership transfer
    pub fn transfer_ticket(
        env: Env,
        ticket_id: String,
        new_owner: Address,
        integrity_hash: String,
    ) {
        new_owner.require_auth();

        let key = DataKey::Ticket(ticket_id.clone());
        let mut ticket: TicketInfo = env.storage().persistent()
            .get(&key)
            .unwrap_or_else(|| panic!("{}", ERR_TICKET_NOT_FOUND));

        if !ticket.for_sale {
            panic!("{}", ERR_NOT_FOR_SALE);
        }

        if ticket.is_used {
            panic!("{}", ERR_TICKET_USED);
        }

        let previous_owner = ticket.owner.clone();

        // Transfer ownership to the buyer
        ticket.owner = new_owner.clone();
        ticket.for_sale = false;
        ticket.resale_price = 0;
        // Update integrity hash to reflect new ownership
        ticket.integrity_hash = integrity_hash;

        env.storage().persistent().set(&key, &ticket);

        env.events().publish(
            (symbol_short!("transfer"), ticket_id),
            (previous_owner, new_owner),
        );
    }

    // ──────────────────────────────────────────
    // GET_TICKET: Read ticket data (view function)
    // ──────────────────────────────────────────

    /// Returns the full TicketInfo record for a given ticket_id.
    /// Used by frontends, gate apps, and verifiers.
    pub fn get_ticket(env: Env, ticket_id: String) -> TicketInfo {
        let key = DataKey::Ticket(ticket_id);
        env.storage().persistent()
            .get(&key)
            .unwrap_or_else(|| panic!("{}", ERR_TICKET_NOT_FOUND))
    }

    // ──────────────────────────────────────────
    // GET_TICKET_COUNT: How many tickets have been issued
    // ──────────────────────────────────────────

    /// Returns the total number of tickets issued by this contract.
    pub fn get_ticket_count(env: Env) -> u32 {
        env.storage().persistent()
            .get(&DataKey::TicketCount)
            .unwrap_or(0)
    }
}