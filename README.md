# Leaderboard Solana

Program ID: GDyukzRzBaZTh9EHuTMmJQwsR6HWuQKQN9BVk6v8xEfn

6/6 tests passing.

# 🏆 On-Chain Leaderboard — Solana Program

A traditional leaderboard/ranking engine rebuilt as a Solana program using Rust and Anchor.  
This project demonstrates how familiar Web2 backend patterns can be reframed using Solana as a **distributed state-machine backend**.

---

## Deployment

| Item | Value |
|---|---|
| Program ID | `GDyukzRzBaZTh9EHuTMmJQwsR6HWuQKQN9BVk6v8xEfn` |
| Cluster | Localnet (Devnet faucet unavailable at submission time) |
| Anchor Version | 0.32.1 |
| Solana CLI | 3.1.11 |
| Rust | 1.94.0 |

> The program was fully deployed and tested on a local validator (`solana-test-validator`).  
> All 6 integration tests pass. Deploy to Devnet requires only `anchor deploy` once faucet SOL is available.

---

## Test Results

```
  leaderboard
✅ Leaderboard initialized: Top Players       (160ms)
✅ Score submitted: Alice = 1000              (431ms)
✅ Score updated: Bob = 2000                  (854ms)
✅ Leaderboard is sorted correctly
✅ Player removed successfully                (426ms)
✅ Leaderboard reset successfully             (435ms)

  6 passing (2s)
```

---

## Architecture Analysis: Web2 → Solana

### How a Leaderboard works in Web2

In a traditional Web2 backend, a leaderboard is a straightforward CRUD service:

```
Client → REST API → Business Logic → Database (Postgres/Redis) → Response
```

**Typical stack:**
- A `scores` table in PostgreSQL with columns: `player_id`, `username`, `score`, `created_at`
- A REST endpoint `POST /scores` validates auth (JWT), checks if player exists, upserts the score
- A `GET /leaderboard` endpoint runs `SELECT ... ORDER BY score DESC LIMIT 10`
- Redis is often used as a sorted set (`ZADD`, `ZRANGE`) for real-time rankings
- An admin dashboard calls `DELETE /scores/:id` or `POST /reset`

**Trust model:** You trust the server. The server trusts the database. Users trust the company not to tamper with data.

---

### How this works on Solana

On Solana, the program *is* the backend. There is no server, no database, no auth token — only on-chain state and cryptographic identity.

```
Client → Solana RPC → Program (Rust) → Account (on-chain state)
```

**Key concepts mapped:**

| Web2 Concept | Solana Equivalent |
|---|---|
| Database row | Account (a blob of bytes owned by the program) |
| Table schema | Anchor `#[account]` struct |
| Primary key | PDA (Program Derived Address) — deterministic, derived from seeds |
| Auth middleware (JWT) | `Signer` constraint — the wallet signs the transaction |
| Admin role check | `has_one = admin` constraint on the account |
| `INSERT OR UPDATE` | Rust logic inside the instruction handler |
| `ORDER BY score DESC` | In-program `Vec::sort_by` after each write |
| `DELETE` | `retain()` to filter the entries vec |

**Account model:**
```
Leaderboard PDA
├── admin: Pubkey          → who controls this board
├── name: String           → leaderboard identifier
├── max_entries: u8        → hard cap (replaces LIMIT in SQL)
├── entries: Vec<Entry>    → sorted list, always kept ordered
│   ├── player: Pubkey
│   ├── player_name: String
│   ├── score: u64
│   └── timestamp: i64
```

The PDA is derived from `["leaderboard", admin_pubkey, name]` — meaning each admin can create multiple named leaderboards, and the address is fully deterministic and verifiable by anyone.

---

### Tradeoffs & Constraints

| | Web2 | Solana |
|---|---|---|
| **Trust** | Trust the company | Trustless — code is law, on-chain |
| **Transparency** | Private DB | Anyone can read the account |
| **Cost** | Server costs (monthly) | Rent per byte stored (~0.002 SOL for 10 entries) |
| **Speed** | <10ms query | ~400ms transaction confirmation |
| **Storage** | Unlimited (scale DB) | Account size fixed at init time |
| **Sorting** | DB index (free) | In-program sort (costs compute units) |
| **Auth** | JWT / session token | Wallet signature (ed25519) |
| **Upgrades** | Deploy anytime | Program upgrade requires authority key |

**Key constraint:** The account size must be pre-allocated at initialization. This is the most important mental shift from Web2 — you can't `ALTER TABLE` on Solana. You must plan storage upfront.

---

## Instructions

| Instruction | Who can call | Description |
|---|---|---|
| `initialize` | Anyone (becomes admin) | Creates a new leaderboard PDA |
| `submit_score` | Admin | Submit or update a player's score |
| `remove_player` | Admin only | Remove a player from the board |
| `reset` | Admin only | Clear all entries |

---

## Getting Started

### Prerequisites

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
sudo apt install -y build-essential

# Install Solana CLI
sh -c "$(curl -sSfL https://release.anza.xyz/stable/install)"
export PATH="/home/$USER/.local/share/solana/install/active_release/bin:$PATH"

# Install Anchor
cargo install --git https://github.com/coral-xyz/anchor avm --locked
avm install latest && avm use latest

# Install Node deps
npm install -g yarn
yarn install
```

### Run Tests Locally

```bash
# Terminal 1 — start local validator
solana-test-validator

# Terminal 2 — run tests
solana config set --url localhost
anchor test --skip-local-validator
```

### Deploy to Devnet

```bash
solana config set --url devnet
solana airdrop 2
anchor build
anchor deploy
```

### Use the CLI

```bash
# Initialize a leaderboard
ts-node client/client.ts init "MyGame" 10

# Submit scores
ts-node client/client.ts submit "MyGame" "Alice"   9500
ts-node client/client.ts submit "MyGame" "Bob"     8200
ts-node client/client.ts submit "MyGame" "Charlie" 9750

# View the leaderboard
ts-node client/client.ts view "MyGame"

# Admin: remove a player
ts-node client/client.ts remove "MyGame" <player-pubkey>

# Admin: reset
ts-node client/client.ts reset "MyGame"
```

**Expected output of `view`:**
```
🏆 Leaderboard: MyGame
   Admin      : H1keMWAnkuwy8bAd6pAbSyknXyHVbNz5u2fUuTRcrstt
   Max entries: 10
   Entries    : 3

  Rank  Player       Score        Date
  ────  ───────────  ───────────  ──────────────────
  #1    Charlie           9750    2026-03-13 14:00:01
  #2    Alice             9500    2026-03-13 13:59:45
  #3    Bob               8200    2026-03-13 13:59:52
```

---

## Project Structure

```
leaderboard/
├── programs/leaderboard/
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs          ← Anchor program (all on-chain logic)
├── tests/
│   └── leaderboard.ts      ← 6 integration tests (all passing)
├── Anchor.toml
├── package.json
├── tsconfig.json
└── README.md
```

---

## Why this matters for Web2 developers

A leaderboard is one of the most universally understood backend systems. By rebuilding it on Solana, this project makes the following concrete:

1. **Solana is a state machine, not just a payment network.** Any system needing shared, tamper-proof state can run here.
2. **Accounts replace tables.** Data is modeled as fixed-size structs, not flexible schemas.
3. **PDAs replace primary keys.** Addresses are deterministic and derivable by anyone.
4. **Wallets replace auth.** No passwords, no JWTs, no sessions. Sign with your key.
5. **The program is the API.** No HTTP layer, no controller, no middleware. Logic lives on-chain.

This mental model unlocks on-chain backends for gaming, DeFi, governance, and any domain where trustless shared state has value.
