[![CI](https://github.com/GabrielGauss/IRL-engine-AX/actions/workflows/ci.yml/badge.svg)](https://github.com/GabrielGauss/IRL-engine-AX/actions/workflows/ci.yml)
![version](https://img.shields.io/badge/version-v1.2.0-0a0a0a?style=flat-square)
![rust edition](https://img.shields.io/badge/rust-2021_edition-b7410e?style=flat-square&logo=rust)
![license](https://img.shields.io/badge/license-MIT-2d6a4f?style=flat-square)
![sandbox](https://img.shields.io/badge/sandbox-live-1a7f37?style=flat-square)
[![PyPI](https://img.shields.io/pypi/v/irl-sdk?style=flat-square&label=irl-sdk%20(python)&color=3776ab)](https://pypi.org/project/irl-sdk/)
[![npm](https://img.shields.io/npm/v/irl-sdk?style=flat-square&label=irl-sdk%20(npm)&color=cc3534)](https://www.npmjs.com/package/irl-sdk)

# IRL Engine

**Immutable Reasoning Log** — a pre-execution compliance gateway that cryptographically seals every autonomous trading decision before it reaches an exchange.

Autonomous AI agents make trading decisions faster than any human oversight mechanism can follow. IRL does not slow them down. It inserts a cryptographic checkpoint between intent and execution: the agent's complete reasoning state is hashed, the decision is evaluated against verified market regime data, and the result is recorded in a tamper-evident audit chain before a single order is submitted. When regulators, risk officers, or counterparties need to reconstruct what the agent knew and why it acted, the proof is already there.

---

## How It Works

The authorize → bind chain ties agent reasoning to exchange execution through a seven-step sequence. Steps 1–4 happen before any order is placed; steps 5–7 close the chain after the exchange confirms.

1. **Register** — Agent submits model hash, notional cap, and permitted regime set to `POST /irl/agents`. IRL creates an entry in the Multi-Agent Registry (MAR).

2. **Authorize** — Before placing any order, the agent calls `POST /irl/authorize` with a complete `CognitiveSnapshot`: model identity, hyperparameter checksum, prompt version, feature schema, action intent, asset, venue, quantity, and notional.

3. **Verify identity** — IRL checks the model hash and agent status against the MAR. Unknown or suspended agents are rejected immediately.

4. **Evaluate policy** — The policy engine checks the requested action against the agent's permitted regime set and notional cap, scaled by the current regime's `max_notional_scale` from the MTA. If any constraint is violated, the request is denied.

5. **Seal the snapshot** — IRL computes `reasoning_hash = SHA-256(RFC 8785 canonical JSON of the full snapshot)` and returns it to the agent. The agent embeds this hash in the exchange order metadata before submission.

6. **Place the order** — The agent places the order through its normal exchange pathway. IRL is not in the execution path.

7. **Bind execution** — The agent calls `POST /irl/bind-execution` with the exchange `tx_id`. IRL computes `final_proof = SHA-256(reasoning_hash ‖ exchange_tx_id)`, reconciles executed parameters against sealed intent, and records a permanent `MATCHED`, `DIVERGENT`, or `EXPIRED` verdict. The audit chain is closed.

Traces are written with bitemporal timestamps (`valid_time` + `transaction_time`) and are never deleted. Every record is final.

---

## Deployment Tiers

| | L1 · $500 / agent / mo | L2 · $1,200 / agent / mo | L3 · Enterprise |
|---|---|---|---|
| Multi-Agent Registry | yes | yes | yes |
| Pre-execution authorize | yes | yes | yes |
| Post-trade bind + verdict | yes | yes | yes |
| Immutable bitemporal trace log | yes | yes | yes |
| Daily Merkle anchor (Bitcoin / OpenTimestamps) | yes | yes | yes |
| Prometheus metrics | yes | yes | yes |
| GDPR purge endpoint | yes | yes | yes |
| Ed25519 signed MTA heartbeats | — | yes | yes |
| Anti-replay sequence IDs | — | yes | yes |
| `mta_ref` verification per trace | — | yes | yes |
| TEE execution attestation | — | — | roadmap |
| Wasm policy modules | — | — | roadmap |
| ZK compliance proofs | — | — | roadmap |

L1 is the core audit rail. It runs standalone with no external dependencies beyond PostgreSQL and is operational within a single day. L2 adds cryptographic regime binding: every authorize call is tied to a specific signed heartbeat from the MTA, preventing regime spoofing and replay. L3 is an enterprise roadmap item targeting TEE-hosted execution and verifiable proofs for submission to regulators without exposing proprietary model state.

---

## Market Truth Anchor (MTA) Interface

IRL is signal-agnostic. It does not hardcode any regime taxonomy, signal provider, or market classification scheme. The MTA is an abstraction over any cryptographically signed source of regime state.

| `MTA_MODE` | Description |
|---|---|
| `macropulse` | Default. Connects to `https://api.macropulse.live`. Verifies Ed25519 signatures on every heartbeat. Caches regime state and tracks sequence IDs. |
| `custom` | Implement the `MtaClient` trait in Rust. IRL reads exactly three normalized fields: `risk_level: f64` (0.0 = defensive, 1.0 = risk-on), `max_notional_scale: f64` (regime multiplier on agent notional cap), `allowed_sides: Vec<String>`. All internal model logic and regime labels remain private to the operator. |
| `none` | Pure audit rail. No regime signal. All sides permitted; agent-level notional caps from the MAR are still enforced. Traces record `signal_mode = "none"`. Appropriate for firms that manage regime risk through an OMS or pre-trade risk system. |
| `mock` | Evaluation and CI mode. No external endpoint required. Fixed permissive regime state. |

In `macropulse` mode and `custom` mode with `LAYER2_ENABLED=true`, each authorize call must include an `mta_ref` — a sequence ID from a recent signed heartbeat. IRL verifies that the heartbeat is current and has not been replayed. Staleness and replay are rejected before the policy evaluation runs.

---

## Quick Start

### Docker Standalone (no external dependencies)

The standalone compose file bundles PostgreSQL and a mock MTA. No MacroPulse account or external service is required.

```bash
git clone https://github.com/GabrielGauss/IRL-engine-AX.git
cd IRL-engine-AX

docker compose -f docker-compose.standalone.yml up -d
```

IRL is available at `http://localhost:4000`. Swagger UI at `http://localhost:4000/swagger-ui/`.

### Cargo Build

```bash
cp .env.example .env
# Minimum required: DATABASE_URL and IRL_API_TOKENS
# For evaluation: MTA_MODE=mock and LAYER2_ENABLED=false

cargo build --release
./target/release/irl-engine
```

PostgreSQL 14+ is required. Run `sqlx migrate run` against the target database before first start, or set `AUTO_MIGRATE=true`.

---

## Environment Variables

| Variable | Required | Default | Description |
|---|---|---|---|
| `DATABASE_URL` | yes | — | PostgreSQL connection string |
| `IRL_API_TOKENS` | yes | — | Comma-separated bearer tokens for API authentication |
| `MTA_MODE` | no | `macropulse` | `macropulse`, `custom`, `none`, or `mock` |
| `MTA_URL` | if `macropulse` | — | MTA operator endpoint (e.g. `https://api.macropulse.live`) |
| `MTA_PUBKEY_HEX` | if `macropulse` | — | Ed25519 public key, 64 hex characters |
| `LAYER2_ENABLED` | no | `true` | Require signed MTA heartbeat reference on every authorize call |
| `SHADOW_MODE` | no | `false` | Audit only — log and seal every request but never block. Safe for dry-run evaluation against production traffic |
| `BIND_SIZE_TOLERANCE` | no | `0.0001` | Quantity divergence tolerance before recording `DIVERGENT` (default: 0.01%) |
| `TRACE_EXPIRY_MS` | no | `3600000` | Time before an unbound `PENDING` trace is marked `EXPIRED` (default: 1 hour) |
| `KMS_PROVIDER` | no | `none` | `local` (AES-256 DEK envelope encryption) or `none` |
| `LOCAL_KMS_KEY` | if `local` | — | 32-byte hex key for local KMS |
| `KMS_KEY_VERSION` | if `local` | — | Key version label |
| `AUTO_MIGRATE` | no | `false` | Run database migrations on startup |
| `PORT` | no | `4000` | HTTP listen port |

---

## SDK Examples

### Python

```bash
pip install irl-sdk
```

```python
import asyncio
from irl_sdk import IRLClient, AuthorizeRequest, TradeAction, OrderType

async def run():
    async with IRLClient(
        irl_url="https://irl.macropulse.live",
        api_token="your-token",
    ) as client:
        result = await client.authorize(AuthorizeRequest(
            agent_id="your-agent-uuid",
            model_id="my-model-v1",
            model_hash_hex="your-model-sha256",
            action=TradeAction.LONG,
            asset="BTC-USD",
            order_type=OrderType.MARKET,
            venue_id="coinbase",
            quantity=0.1,
            notional=6500.0,
            notional_currency="USD",
        ))

        assert result.authorized
        # Embed result.reasoning_hash in exchange order metadata before submitting

        tx_id = await exchange.place_order(reasoning_hash=result.reasoning_hash)

        await client.bind_execution(
            trace_id=result.trace_id,
            exchange_tx_id=tx_id,
            execution_status="Filled",
            asset="BTC-USD",
            executed_quantity=0.1,
            execution_price=65000.0,
        )

asyncio.run(run())
```

### TypeScript

```bash
npm install irl-sdk
```

```ts
import { IRLClient } from "irl-sdk";

const client = new IRLClient({
  irlUrl: "https://irl.macropulse.live",
  apiToken: process.env.IRL_API_TOKEN!,
});

const result = await client.authorize({
  agent_id: "your-agent-uuid",
  model_id: "my-model-v1",
  model_hash_hex: "your-model-sha256",
  action: "Long",
  asset: "BTC-USD",
  venue_id: "CBSE",
  quantity: 0.1,
  notional: 6500,
});

if (result.authorized) {
  // Embed result.reasoning_hash in exchange order metadata before submitting
  const txId = await exchange.placeOrder({ reasoning_hash: result.reasoning_hash });

  await client.bindExecution({
    trace_id: result.trace_id,
    exchange_tx_id: txId,
    execution_status: "Filled",
    asset: "BTC-USD",
    executed_quantity: 0.1,
    execution_price: 65000,
  });
}

await client.close();
```

---

## API Reference

All endpoints except `/irl/health` require `Authorization: Bearer <token>`.

| Method | Route | Description |
|---|---|---|
| `GET` | `/irl/health` | Liveness check |
| `POST` | `/irl/agents` | Register an agent (model hash, notional cap, regime permissions) |
| `GET` | `/irl/agents` | List all registered agents |
| `GET` | `/irl/agents/:id` | Retrieve agent profile |
| `PATCH` | `/irl/agents/:id/status` | Suspend or deregister an agent |
| `POST` | `/irl/authorize` | Seal a CognitiveSnapshot, receive `reasoning_hash` |
| `POST` | `/irl/bind-execution` | Bind exchange confirmation, receive `final_proof` |
| `GET` | `/irl/trace/:id` | Full audit record for a trace (forensic replay) |
| `GET` | `/irl/pending` | Traces awaiting bind-execution |
| `GET` | `/irl/orphans` | `DIVERGENT` and `EXPIRED` traces |
| `DELETE` | `/irl/gdpr/purge/:agent_id` | GDPR Article 17 erasure request |
| `GET` | `/metrics` | Prometheus metrics endpoint |

Full request and response schemas are available at the sandbox Swagger UI: `https://irl.macropulse.live/swagger-ui/`

---

## Architecture

IRL is a single Axum 0.7 service backed by PostgreSQL. All components run in-process.

| Component | Source | Responsibility |
|---|---|---|
| **Multi-Agent Registry (MAR)** | `registry.rs` | Agent lifecycle: model hash pinning, notional caps, regime permissions, active/suspended status |
| **Policy engine** | `policy.rs` | Evaluates each authorize request against MAR constraints and current MTA regime state |
| **Seal module** | `seal.rs` | RFC 8785 canonical JSON serialization + SHA-256 hashing of CognitiveSnapshot |
| **Heartbeat verifier** | `heartbeat.rs` | Ed25519 signature verification, sequence ID tracking, anti-replay enforcement (L2) |
| **Snapshot store** | `snapshot.rs` | Bitemporal persistence of CognitiveSnapshot records |
| **Binding verifier** | `binding.rs` | Post-trade reconciliation, `final_proof` computation, `MATCHED`/`DIVERGENT`/`EXPIRED` verdict |
| **Merkle anchor** | `merkle.rs` | Daily OpenTimestamps anchoring of the audit log leaf set to Bitcoin |
| **KMS layer** | `kms.rs` | AWS KMS, HashiCorp Vault, or local AES-256 DEK envelope encryption |
| **Shadow mode** | `shadow_mode.rs` | Intercepts policy decisions and converts blocks to audit-only observations |
| **Backfill** | `backfill.rs` | Replay and re-seal historical snapshots for migration or forensic reconstruction |
| **GDPR** | `gdpr.rs` | Right-to-erasure handler; soft-purges agent data while preserving audit integrity |
| **Token manager** | `token_manager.rs` | Bearer token issuance, rotation, and revocation |
| **Metrics** | `metrics.rs` | Prometheus counters and histograms for authorize latency, bind rate, divergence rate, MAR size |

---

## Compliance Mapping

| Regulation | Provision | How IRL addresses it |
|---|---|---|
| **MiFID II Article 17** | Algorithmic trading — organisational requirements and pre-trade controls | Pre-execution authorization gate with cryptographic proof of intent; full audit trail per decision; model hash pinning ties each trace to a specific deployed model version |
| **EU AI Act** | High-risk AI system obligations — transparency, traceability, human oversight capability | Immutable, bitemporal trace log; CognitiveSnapshot records complete epistemic state; Merkle anchoring provides tamper-evidence independent of the IRL operator |
| **SEC Rule 15c3-5** | Market Access Rule — pre-trade risk controls for broker-dealers | Notional cap enforcement per agent per regime; side restrictions enforced before any order is placed; audit log available for regulatory examination |
| **DORA** | Digital Operational Resilience Act — ICT risk and incident reporting | Prometheus metrics for operational visibility; shadow mode for resilience testing without disrupting live control; bitemporal records support incident reconstruction |

---

## Ecosystem

| Resource | URL |
|---|---|
| Sandbox | `https://irl.macropulse.live` |
| Swagger UI | `https://irl.macropulse.live/swagger-ui/` |
| Public documentation | `https://github.com/GabrielGauss/irl-public-docs` |
| MacroPulse (MTA source) | `https://macropulse.live` · API: `https://api.macropulse.live` |
| Python SDK | [`irl-sdk` on PyPI](https://pypi.org/project/irl-sdk/) · [source](https://github.com/GabrielGauss/irl-sdk-python) |
| TypeScript SDK | [`irl-sdk` on npm](https://www.npmjs.com/package/irl-sdk) · [source](https://github.com/GabrielGauss/irl-sdk-ts) |

---

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). Bug reports and feature requests via [GitHub Issues](https://github.com/GabrielGauss/IRL-engine-AX/issues).

---

## License

MIT. See [LICENSE](LICENSE).

Commercial licensing, enterprise deployment, and custom MTA integration support: hello@macropulse.live
