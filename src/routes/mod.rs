pub mod admin;
pub mod agents;
pub mod authorize;
pub mod bind;
pub mod tokens;
pub mod traces;

use crate::db;
use crate::errors::AppError;
use crate::metrics;
use crate::AppState;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse},
    Json,
};
use serde::Deserialize;
use uuid::Uuid;

/// GET /
///
/// HTML landing page for the demo/sandbox instance.
/// Links to Swagger UI, health endpoint, and documentation.
pub async fn landing() -> Html<&'static str> {
    Html(LANDING_HTML)
}

const LANDING_HTML: &str = r##"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>IRL — Prove what your AI agent decided, before it trades</title>
<meta name="description" content="IRL (Immutable Reasoning Log) seals an AI trading agent's reasoning before any order reaches the exchange, anchors it to Bitcoin, and lets anyone verify it offline. Pre-execution compliance for autonomous trading.">
<link rel="icon" type="image/svg+xml" href="https://macropulse.live/favicon.svg">
<meta property="og:title" content="IRL — Prove what your AI agent decided, before it trades">
<meta property="og:description" content="Cryptographic pre-execution compliance for autonomous trading agents. Sealed before execution, anchored to Bitcoin, verifiable by anyone — without trusting us.">
<meta property="og:image" content="https://macropulse.live/og-image.png">
<meta property="og:url" content="https://irl.macropulse.live/">
<meta property="og:type" content="website">
<meta name="twitter:card" content="summary_large_image">
<meta name="twitter:title" content="IRL — Prove what your AI agent decided, before it trades">
<meta name="twitter:description" content="Pre-execution compliance for autonomous trading agents. Sealed, anchored to Bitcoin, verifiable offline.">
<meta name="twitter:image" content="https://macropulse.live/og-image.png">
<link rel="preconnect" href="https://fonts.googleapis.com">
<link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
<link href="https://fonts.googleapis.com/css2?family=Inter:wght@300;400;500;600;700;800&family=JetBrains+Mono:wght@400;500;600&display=swap" rel="stylesheet">
<style>
  *, *::before, *::after { box-sizing: border-box; margin: 0; padding: 0; }
  :root {
    --bg: #090909; --s1: #0f0f0f; --s2: #141414;
    --border: #1a1a1a; --border2: #262626;
    --text: #f2f2f2; --muted: #8a8a8a; --dim: #555;
    --amber: #f5a623; --amber-dim: rgba(245,166,35,0.1); --amber-brd: rgba(245,166,35,0.32);
    --green: #3fb85a; --green-dim: rgba(63,184,90,0.1);
    --blue: #7dd3fc; --grn: #86efac; --str: #fca5a5; --pur: #c4b5fd;
    --sans: 'Inter', -apple-system, sans-serif; --mono: 'JetBrains Mono', monospace;
  }
  html { scroll-behavior: smooth; }
  body { background: var(--bg); color: var(--text); font-family: var(--sans); line-height: 1.6; -webkit-font-smoothing: antialiased; overflow-x: hidden; }
  a { color: inherit; text-decoration: none; }
  .mono { font-family: var(--mono); }
  ::selection { background: rgba(245,166,35,0.3); }

  /* Nav */
  nav { position: fixed; top: 0; left: 0; right: 0; z-index: 100; height: 58px; display: flex; align-items: center; justify-content: space-between; padding: 0 2rem; border-bottom: 1px solid var(--border); background: rgba(9,9,9,0.9); backdrop-filter: blur(16px); }
  .nav-brand { font-size: 0.95rem; font-weight: 700; letter-spacing: -0.01em; display: flex; align-items: center; gap: 0.55rem; }
  .nav-brand .sub { color: var(--muted); font-weight: 500; }
  .nav-links { display: flex; align-items: center; gap: 1.6rem; }
  .nav-links a { font-size: 0.82rem; color: var(--muted); transition: color 0.2s; }
  .nav-links a:hover { color: var(--text); }
  .nav-cta { padding: 0.45rem 1rem; background: var(--amber); color: #0a0a0a !important; border-radius: 6px; font-size: 0.8rem; font-weight: 600; }
  .nav-cta:hover { filter: brightness(1.08); }
  @media (max-width: 820px) { .nav-links .hide-sm { display: none; } nav { padding: 0 1.1rem; } }

  .wrap { max-width: 1000px; margin: 0 auto; padding: 0 2rem; }
  .narrow { max-width: 760px; }
  @media (max-width: 640px) { .wrap { padding: 0 1.25rem; } }

  /* Hero */
  .hero { padding: 9rem 0 4.5rem; position: relative; }
  .hero::before { content: ''; position: absolute; top: -10%; left: 50%; transform: translateX(-50%); width: 900px; height: 600px; background: radial-gradient(ellipse, rgba(245,166,35,0.08), transparent 60%); pointer-events: none; z-index: 0; }
  .hero > * { position: relative; z-index: 1; }
  .eyebrow { display: inline-flex; align-items: center; gap: 0.5rem; font-family: var(--mono); font-size: 0.72rem; color: var(--amber); letter-spacing: 0.08em; text-transform: uppercase; font-weight: 600; background: var(--amber-dim); border: 1px solid var(--amber-brd); padding: 0.3rem 0.8rem; border-radius: 5px; margin-bottom: 1.75rem; }
  .hero h1 { font-size: clamp(2.1rem, 5.5vw, 3.7rem); font-weight: 800; letter-spacing: -0.04em; line-height: 1.05; margin-bottom: 1.35rem; max-width: 15ch; }
  .hero h1 em { font-style: normal; color: var(--amber); }
  .hero-sub { font-size: clamp(1rem, 1.6vw, 1.15rem); color: var(--muted); max-width: 620px; line-height: 1.65; margin-bottom: 2.25rem; }
  .hero-actions { display: flex; gap: 0.8rem; flex-wrap: wrap; align-items: center; }
  .btn { display: inline-flex; align-items: center; gap: 0.4rem; border-radius: 7px; font-size: 0.9rem; font-weight: 600; padding: 0.75rem 1.4rem; transition: all 0.18s; cursor: pointer; border: none; font-family: var(--sans); }
  .btn-primary { background: var(--amber); color: #0a0a0a; }
  .btn-primary:hover { filter: brightness(1.08); transform: translateY(-1px); }
  .btn-ghost { border: 1px solid var(--border2); color: var(--muted); background: transparent; }
  .btn-ghost:hover { border-color: var(--muted); color: var(--text); }
  .trust-strip { display: flex; gap: 1.5rem; flex-wrap: wrap; margin-top: 2.5rem; font-family: var(--mono); font-size: 0.75rem; color: var(--dim); }
  .trust-strip span { display: inline-flex; align-items: center; gap: 0.45rem; }
  .trust-strip .dot { color: var(--amber); }

  /* Section scaffolding */
  section { padding: 4.5rem 0; border-top: 1px solid var(--border); }
  .s-label { font-family: var(--mono); font-size: 0.72rem; color: var(--amber); letter-spacing: 0.1em; text-transform: uppercase; font-weight: 600; margin-bottom: 0.9rem; }
  h2 { font-size: clamp(1.5rem, 3vw, 2.1rem); font-weight: 700; letter-spacing: -0.03em; line-height: 1.15; margin-bottom: 1rem; }
  .lead { font-size: 1.02rem; color: var(--muted); max-width: 640px; line-height: 1.7; }

  /* Problem */
  .prob-grid { display: grid; grid-template-columns: repeat(3, 1fr); gap: 1.1rem; margin-top: 2.5rem; }
  @media (max-width: 760px) { .prob-grid { grid-template-columns: 1fr; } }
  .prob { background: var(--s1); border: 1px solid var(--border2); border-radius: 10px; padding: 1.5rem; }
  .prob .n { font-family: var(--mono); font-size: 0.7rem; color: var(--str); margin-bottom: 0.7rem; letter-spacing: 0.05em; }
  .prob h3 { font-size: 1rem; font-weight: 600; margin-bottom: 0.5rem; }
  .prob p { font-size: 0.87rem; color: var(--muted); line-height: 1.6; }

  /* How it works */
  .flow { display: grid; grid-template-columns: repeat(3, 1fr); gap: 1.1rem; margin-top: 2.5rem; }
  @media (max-width: 760px) { .flow { grid-template-columns: 1fr; } }
  .flow-step { background: linear-gradient(180deg, var(--amber-dim), transparent); border: 1px solid var(--amber-brd); border-radius: 12px; padding: 1.6rem; position: relative; }
  .flow-step .step-n { font-family: var(--mono); font-size: 0.72rem; color: var(--amber); font-weight: 600; margin-bottom: 0.85rem; }
  .flow-step h3 { font-size: 1.05rem; font-weight: 700; margin-bottom: 0.5rem; letter-spacing: -0.01em; }
  .flow-step p { font-size: 0.87rem; color: var(--muted); line-height: 1.6; }
  .flow-step code { font-family: var(--mono); font-size: 0.76rem; color: var(--amber); background: rgba(245,166,35,0.08); padding: 0.1rem 0.35rem; border-radius: 4px; }

  /* Verify (differentiator) */
  .verify { background: var(--s1); border: 1px solid var(--border2); border-radius: 16px; padding: clamp(1.75rem, 4vw, 3rem); margin-top: 1rem; }
  .verify h2 { margin-bottom: 0.75rem; }
  .verify .big { font-size: clamp(1.15rem, 2.4vw, 1.5rem); font-weight: 600; letter-spacing: -0.02em; margin: 1.75rem 0 0.4rem; }
  .verify .big .a { color: var(--amber); }
  .verify-actions { display: flex; gap: 0.75rem; flex-wrap: wrap; margin-top: 1.75rem; }
  .chip { display: inline-flex; align-items: center; gap: 0.45rem; font-family: var(--mono); font-size: 0.78rem; color: var(--muted); border: 1px solid var(--border2); border-radius: 6px; padding: 0.5rem 0.9rem; transition: border-color 0.2s, color 0.2s; }
  .chip:hover { border-color: var(--amber-brd); color: var(--text); }

  /* Proof / MacroPulse origin */
  .proof { display: flex; gap: 1.5rem; align-items: flex-start; margin-top: 1rem; padding: 1.75rem; border: 1px solid rgba(63,184,90,0.25); border-radius: 14px; background: var(--green-dim); }
  .proof .mark { flex-shrink: 0; }
  .proof h3 { font-size: 1.05rem; font-weight: 700; margin-bottom: 0.5rem; }
  .proof p { font-size: 0.9rem; color: var(--muted); line-height: 1.65; }
  .proof a.inline { color: var(--green); border-bottom: 1px solid rgba(63,184,90,0.4); }
  @media (max-width: 560px) { .proof { flex-direction: column; gap: 1rem; } }

  /* Regulatory */
  .reg-grid { display: grid; grid-template-columns: repeat(4, 1fr); gap: 0.9rem; margin-top: 2rem; }
  @media (max-width: 760px) { .reg-grid { grid-template-columns: repeat(2, 1fr); } }
  .reg { border: 1px solid var(--border2); border-radius: 9px; padding: 1.1rem 1.25rem; background: var(--s1); }
  .reg .name { font-family: var(--mono); font-size: 0.82rem; font-weight: 600; color: var(--text); margin-bottom: 0.3rem; }
  .reg .what { font-size: 0.76rem; color: var(--muted); line-height: 1.5; }

  /* Pricing */
  .price-grid { display: grid; grid-template-columns: repeat(3, 1fr); gap: 1.1rem; margin-top: 2.5rem; }
  @media (max-width: 820px) { .price-grid { grid-template-columns: 1fr; } }
  .tier { border: 1px solid var(--border2); border-radius: 13px; padding: 1.75rem; background: var(--s1); display: flex; flex-direction: column; }
  .tier.featured { border-color: var(--amber-brd); background: linear-gradient(180deg, var(--amber-dim), var(--s1)); box-shadow: 0 0 50px -25px rgba(245,166,35,0.5); }
  .tier .badge { display: inline-block; font-family: var(--mono); font-size: 0.62rem; font-weight: 700; letter-spacing: 0.1em; text-transform: uppercase; color: var(--amber); border: 1px solid var(--amber-brd); background: var(--amber-dim); padding: 0.15rem 0.5rem; border-radius: 4px; margin-bottom: 0.85rem; align-self: flex-start; }
  .tier .tname { font-size: 0.9rem; font-weight: 700; margin-bottom: 0.3rem; }
  .tier .tagline { font-size: 0.8rem; color: var(--muted); margin-bottom: 1rem; min-height: 2.4em; }
  .tier .price { font-family: var(--mono); font-size: 2rem; font-weight: 600; letter-spacing: -0.03em; line-height: 1; }
  .tier .price .per { font-size: 0.78rem; color: var(--muted); font-weight: 400; font-family: var(--sans); }
  .tier .minq { font-family: var(--mono); font-size: 0.66rem; color: var(--dim); text-transform: uppercase; letter-spacing: 0.06em; margin: 0.5rem 0 1.25rem; }
  .tier ul { list-style: none; display: flex; flex-direction: column; gap: 0.55rem; margin-bottom: 1.5rem; flex: 1; }
  .tier li { font-size: 0.83rem; color: var(--muted); display: flex; gap: 0.5rem; align-items: flex-start; line-height: 1.5; }
  .tier li::before { content: '+'; color: var(--amber); font-weight: 700; flex-shrink: 0; }
  .tier li.inherit::before { content: '\21B3'; color: var(--dim); }
  .tier .tcta { text-align: center; padding: 0.65rem; border-radius: 7px; font-size: 0.85rem; font-weight: 600; transition: all 0.18s; }
  .tier .tcta.solid { background: var(--amber); color: #0a0a0a; }
  .tier .tcta.solid:hover { filter: brightness(1.08); }
  .tier .tcta.ghost { border: 1px solid var(--border2); color: var(--muted); }
  .tier .tcta.ghost:hover { border-color: var(--muted); color: var(--text); }
  .price-note { text-align: center; font-size: 0.8rem; color: var(--dim); margin-top: 1.75rem; }
  .price-note a { color: var(--muted); border-bottom: 1px solid var(--border2); }

  /* Sandbox */
  .sandbox { background: var(--s1); border: 1px solid var(--border2); border-radius: 14px; padding: clamp(1.75rem, 4vw, 2.5rem); margin-top: 1rem; }
  .sandbox-top { display: flex; justify-content: space-between; align-items: flex-start; flex-wrap: wrap; gap: 1rem; margin-bottom: 1.5rem; }
  .sandbox pre { background: var(--bg); border: 1px solid var(--border2); border-radius: 8px; padding: 1rem 1.15rem; font-family: var(--mono); font-size: 0.76rem; overflow-x: auto; line-height: 1.65; margin-top: 1rem; }
  .comment { color: var(--dim); } .key { color: var(--blue); } .val { color: var(--grn); } .str { color: var(--str); } .url { color: var(--pur); }
  .sandbox-links { display: flex; gap: 0.7rem; flex-wrap: wrap; margin-top: 1.5rem; }

  /* Final CTA */
  .final { text-align: center; padding: 5.5rem 0; }
  .final h2 { font-size: clamp(1.8rem, 4vw, 2.6rem); margin-bottom: 1rem; }
  .final p { color: var(--muted); max-width: 500px; margin: 0 auto 2rem; }
  .final .hero-actions { justify-content: center; }

  footer { border-top: 1px solid var(--border); padding: 2rem; }
  .footer-in { max-width: 1000px; margin: 0 auto; display: flex; align-items: center; justify-content: space-between; flex-wrap: wrap; gap: 1rem; }
  .footer-brand { font-size: 0.8rem; color: var(--dim); display: flex; align-items: center; gap: 0.5rem; }
  .footer-links { display: flex; gap: 1.4rem; flex-wrap: wrap; }
  .footer-links a { font-size: 0.8rem; color: var(--dim); transition: color 0.15s; }
  .footer-links a:hover { color: var(--muted); }
</style>
</head>
<body>

<nav>
  <a class="nav-brand" href="/">
    <svg width="22" height="22" viewBox="0 0 100 100" fill="none" style="flex-shrink:0" aria-label="MacroPulse"><defs><clipPath id="lg-cr"><circle cx="54" cy="50" r="32"/></clipPath><mask id="lg-lmr"><circle cx="44" cy="50" r="32" fill="#fff"/><circle cx="54" cy="50" r="32" fill="#000"/></mask></defs><g fill="#3fb85a"><g mask="url(#lg-lmr)"><rect x="0" y="13.5" width="100" height="4.9"/><rect x="0" y="22.5" width="100" height="4.9"/><rect x="0" y="31.5" width="100" height="4.9"/><rect x="0" y="40.5" width="100" height="4.9"/><rect x="0" y="49.5" width="100" height="4.9"/><rect x="0" y="58.5" width="100" height="4.9"/><rect x="0" y="67.5" width="100" height="4.9"/><rect x="0" y="76.5" width="100" height="4.9"/><rect x="0" y="85.5" width="100" height="4.9"/></g><g clip-path="url(#lg-cr)"><rect x="0" y="9" width="100" height="4.9"/><rect x="0" y="18" width="100" height="4.9"/><rect x="0" y="27" width="100" height="4.9"/><rect x="0" y="36" width="100" height="4.9"/><rect x="0" y="45" width="100" height="4.9"/><rect x="0" y="54" width="100" height="4.9"/><rect x="0" y="63" width="100" height="4.9"/><rect x="0" y="72" width="100" height="4.9"/><rect x="0" y="81" width="100" height="4.9"/><rect x="0" y="90" width="100" height="4.9"/></g></g></svg>
    IRL <span class="sub">· MacroPulse</span>
  </a>
  <div class="nav-links">
    <a href="#how" class="hide-sm">How it works</a>
    <a href="#verify" class="hide-sm">Verify</a>
    <a href="#pricing">Pricing</a>
    <a href="#sandbox" class="hide-sm">Sandbox</a>
    <a href="https://github.com/macropulse-lab/irl-public-docs" class="hide-sm">Docs</a>
    <a href="#pricing" class="nav-cta">Get access</a>
  </div>
</nav>

<div class="wrap">
  <header class="hero">
    <div class="eyebrow">Pre-execution compliance for AI trading agents</div>
    <h1>Prove what your AI agent decided — <em>before it trades.</em></h1>
    <p class="hero-sub">
      IRL seals your agent's reasoning the instant before an order reaches the exchange, binds it to the fill,
      and anchors it to Bitcoin. A tamper-evident record of every decision — that anyone can verify offline,
      without trusting you, or us.
    </p>
    <div class="hero-actions">
      <a href="#sandbox" class="btn btn-primary">Try the sandbox — no signup →</a>
      <a href="https://macropulse.live/irl-whitepaper" class="btn btn-ghost">Read the whitepaper</a>
    </div>
    <div class="trust-strip">
      <span><span class="dot">◆</span> Sealed pre-execution</span>
      <span><span class="dot">◆</span> Anchored to Bitcoin</span>
      <span><span class="dot">◆</span> Verifiable offline</span>
    </div>
  </header>
</div>

<!-- Problem -->
<section id="problem">
  <div class="wrap">
    <div class="s-label">The problem</div>
    <h2>An AI agent just made a trade you can't explain.</h2>
    <p class="lead">Autonomous agents decide in milliseconds, faster than any human reviews. When one loses big — or does something that looks like abuse — the only record is a log file you control. That's not a defense.</p>
    <div class="prob-grid">
      <div class="prob">
        <div class="n">01 · UNEXPLAINABLE</div>
        <h3>No reasoning trail</h3>
        <p>The model acted on state that's already gone. Reconstructing "why" after the fact is guesswork a regulator won't accept.</p>
      </div>
      <div class="prob">
        <div class="n">02 · EDITABLE</div>
        <h3>Logs can be rewritten</h3>
        <p>Records that live on your own servers can be changed — so they prove nothing to an auditor, an LP, or a court.</p>
      </div>
      <div class="prob">
        <div class="n">03 · LIABILITY</div>
        <h3>"Trust us" is the exposure</h3>
        <p>With no independent proof, every automated decision is a liability waiting for a subpoena you can't answer.</p>
      </div>
    </div>
  </div>
</section>

<!-- How it works -->
<section id="how">
  <div class="wrap">
    <div class="s-label">How it works</div>
    <h2>Seal. Bind. Anchor.</h2>
    <p class="lead">IRL runs as a gateway between your agent and the exchange. Three steps turn an automated decision into permanent, checkable evidence.</p>
    <div class="flow">
      <div class="flow-step">
        <div class="step-n">01 — SEAL</div>
        <h3>Before the order</h3>
        <p>Your agent submits its reasoning — model, intent, size, risk checks. IRL runs pre-execution policy, then seals the snapshot with <code>SHA-256</code> (RFC 8785 canonical JSON). Nothing trades until it passes.</p>
      </div>
      <div class="flow-step">
        <div class="step-n">02 — BIND</div>
        <h3>After the fill</h3>
        <p>The exchange returns a transaction ID. IRL computes <code>final_proof = SHA-256(reasoning &#8214; tx_id)</code>, cryptographically binding the sealed intent to what actually executed.</p>
      </div>
      <div class="flow-step">
        <div class="step-n">03 — ANCHOR</div>
        <h3>Every day</h3>
        <p>All seals roll into a daily Merkle root, anchored to Bitcoin via OpenTimestamps. The record becomes tamper-evident against everyone — including us.</p>
      </div>
    </div>
  </div>
</section>

<!-- Verify -->
<section id="verify">
  <div class="wrap">
    <div class="verify">
      <div class="s-label">Trust without trust</div>
      <h2>You don't have to trust us.</h2>
      <p class="lead">Every audit vendor claims a tamper-proof log. Only IRL hands the proof to the other side — the auditor, the regulator, the LP — and lets them check it with zero access to our servers.</p>
      <div class="big"><span class="a">Open-source verifier.</span> Frozen spec. Reimplement it in any language.</div>
      <div class="big">Check any proof bundle <span class="a">offline, against Bitcoin.</span></div>
      <p class="lead" style="margin-top:1rem;">We can't fake it. We can't rewrite it. Neither can you. That's the entire design.</p>
      <div class="verify-actions">
        <a href="https://github.com/macropulse-lab/irl-verify" class="chip">↗ irl-verify (MIT, offline)</a>
        <a href="https://macropulse.live/proof" class="chip">↗ In-browser proof explorer</a>
        <a href="https://github.com/macropulse-lab/irl-public-docs" class="chip">↗ Frozen spec</a>
      </div>
    </div>
  </div>
</section>

<!-- Proof / origin -->
<section id="proof">
  <div class="wrap">
    <div class="s-label">Why us</div>
    <h2>We run our own signal on this discipline.</h2>
    <div class="proof">
      <svg class="mark" width="56" height="56" viewBox="0 0 100 100" fill="none" aria-label="MacroPulse"><defs><clipPath id="pf-cr"><circle cx="54" cy="50" r="32"/></clipPath><mask id="pf-lmr"><circle cx="44" cy="50" r="32" fill="#fff"/><circle cx="54" cy="50" r="32" fill="#000"/></mask></defs><g fill="#3fb85a"><g mask="url(#pf-lmr)"><rect x="0" y="13.5" width="100" height="4.9"/><rect x="0" y="22.5" width="100" height="4.9"/><rect x="0" y="31.5" width="100" height="4.9"/><rect x="0" y="40.5" width="100" height="4.9"/><rect x="0" y="49.5" width="100" height="4.9"/><rect x="0" y="58.5" width="100" height="4.9"/><rect x="0" y="67.5" width="100" height="4.9"/><rect x="0" y="76.5" width="100" height="4.9"/><rect x="0" y="85.5" width="100" height="4.9"/></g><g clip-path="url(#pf-cr)"><rect x="0" y="9" width="100" height="4.9"/><rect x="0" y="18" width="100" height="4.9"/><rect x="0" y="27" width="100" height="4.9"/><rect x="0" y="36" width="100" height="4.9"/><rect x="0" y="45" width="100" height="4.9"/><rect x="0" y="54" width="100" height="4.9"/><rect x="0" y="63" width="100" height="4.9"/><rect x="0" y="72" width="100" height="4.9"/><rect x="0" y="81" width="100" height="4.9"/><rect x="0" y="90" width="100" height="4.9"/></g></g></svg>
      <div>
        <h3>IRL is built by the team behind MacroPulse.</h3>
        <p>MacroPulse is a macro-regime signal we run in production — Ed25519-signed and publicly verifiable, every day. Proving our own market calls is how we learned how hard real proof is. IRL's Layer 2 can bind each decision to that signed market truth. <a class="inline" href="https://macropulse.live/track-record">See the public track record →</a></p>
      </div>
    </div>
  </div>
</section>

<!-- Regulatory -->
<section id="reg">
  <div class="wrap">
    <div class="s-label">Built for what's coming</div>
    <h2>Auditability is becoming law.</h2>
    <p class="lead">The direction of travel is one-way: automated decisions must be explainable and provable. IRL maps to the specific requirements each framework asks for — not vague "auditability."</p>
    <div class="reg-grid">
      <div class="reg"><div class="name">MiFID II · RTS 6</div><div class="what">Algorithmic trading records &amp; pre-trade controls</div></div>
      <div class="reg"><div class="name">EU AI Act</div><div class="what">High-risk system logging &amp; traceability</div></div>
      <div class="reg"><div class="name">SEC 15c3-5</div><div class="what">Pre-trade risk controls on market access</div></div>
      <div class="reg"><div class="name">DORA</div><div class="what">Operational resilience &amp; evidence trails</div></div>
    </div>
  </div>
</section>

<!-- Pricing -->
<section id="pricing">
  <div class="wrap">
    <div class="s-label">Pricing</div>
    <h2>Start proving in a day.</h2>
    <p class="lead">Self-host in your own VPC. Per-agent pricing, unlimited calls. Verification is free for everyone, forever — your auditors never need an account.</p>
    <div class="price-grid">
      <div class="tier featured">
        <span class="badge">L1 · Most common</span>
        <div class="tname">IRL Sidecar</div>
        <div class="tagline">Prove what happened. Wraps any existing agent in ~20 lines.</div>
        <div class="price">$500<span class="per"> / agent / mo</span></div>
        <div class="minq">Min 1 agent · unlimited calls</div>
        <ul>
          <li>Pre-execution enforcement — hard halt before the exchange</li>
          <li>SHA-256 reasoning seal + bitemporal audit ledger</li>
          <li>Multi-agent registry &amp; post-trade verifier</li>
          <li>Daily Merkle anchor to Bitcoin</li>
          <li>Proof-bundle export · Python &amp; TypeScript SDKs</li>
        </ul>
        <a href="https://macropulse.live/irl#pricing" class="tcta solid">Start with L1 →</a>
      </div>
      <div class="tier">
        <span class="badge">L2</span>
        <div class="tname">IRL Audit Platform</div>
        <div class="tagline">Prove what the market was. Signed truth + anti-replay.</div>
        <div class="price">$1,200<span class="per"> / agent / mo</span></div>
        <div class="minq">Min 3 agents · 99.9% SLA</div>
        <ul>
          <li class="inherit">Everything in L1, plus:</li>
          <li>Signed Ed25519 heartbeats — anti-replay</li>
          <li>MacroPulse MTA integration (or your own)</li>
          <li>Forensic replay of any historical trade</li>
          <li>Read-only compliance dashboard</li>
        </ul>
        <a href="https://macropulse.live/irl#pricing" class="tcta ghost">Get L2 →</a>
      </div>
      <div class="tier">
        <span class="badge">L3 · Coming soon</span>
        <div class="tname">Sovereign Gateway</div>
        <div class="tagline">Prove without revealing. For alpha you can't expose.</div>
        <div class="price" style="font-size:1.5rem;">Enterprise</div>
        <div class="minq">TEE + ZK · quoted per engagement</div>
        <ul>
          <li class="inherit">Everything in L2, plus:</li>
          <li>TEE execution — hardware-attested enclave</li>
          <li>ZK compliance proofs — prove without disclosure</li>
          <li>Dedicated support · quarterly review</li>
        </ul>
        <a href="mailto:licensing@macropulse.live?subject=IRL%20L3%20Sovereign%20Gateway" class="tcta ghost">Talk to us →</a>
      </div>
    </div>
    <p class="price-note">Evaluating for a fleet? <a href="mailto:licensing@macropulse.live?subject=IRL%20technical%20demo">Book a technical demo →</a> · Fleet discounts from 6+ agents.</p>
  </div>
</section>

<!-- Sandbox -->
<section id="sandbox">
  <div class="wrap">
    <div class="sandbox">
      <div class="sandbox-top">
        <div>
          <div class="s-label" style="margin-bottom:0.5rem;">For developers</div>
          <h2 style="margin-bottom:0.4rem;">Try it now. No signup.</h2>
          <p class="lead">Three demo agents are pre-seeded. Run a full authorize → bind flow in the interactive API, then verify the proof bundle yourself.</p>
        </div>
      </div>
      <pre><span class="comment"># POST /irl/authorize — seal the reasoning before the order</span>
{
  <span class="key">"agent_id"</span>: <span class="str">"00000000-0000-4000-a000-000000000001"</span>,
  <span class="key">"model_hash"</span>: <span class="str">"sha256-hex-of-model"</span>,
  <span class="key">"action"</span>: { <span class="key">"Long"</span>: <span class="val">1.5</span> },
  <span class="key">"asset"</span>: <span class="str">"BTC/USD"</span>,
  <span class="key">"notional"</span>: <span class="val">64875.00</span>
}
<span class="comment"># → { "trace_id": "...", "reasoning_hash": "sha256...", "authorized": true }</span></pre>
      <div class="sandbox-links">
        <a href="/swagger-ui/" class="btn btn-primary">Open interactive API →</a>
        <a href="/openapi.json" class="btn btn-ghost">OpenAPI JSON</a>
        <a href="/health" class="btn btn-ghost">Health</a>
      </div>
    </div>
  </div>
</section>

<!-- Final CTA -->
<div class="wrap">
  <div class="final">
    <h2>Every agent decision, provable.</h2>
    <p>Start with the sandbox, self-host in a day, and hand your auditors evidence they can verify without trusting you.</p>
    <div class="hero-actions">
      <a href="#pricing" class="btn btn-primary">Get access →</a>
      <a href="/swagger-ui/" class="btn btn-ghost">Try the sandbox</a>
    </div>
  </div>
</div>

<footer>
  <div class="footer-in">
    <div class="footer-brand">
      <svg width="16" height="16" viewBox="0 0 100 100" fill="none"><defs><clipPath id="ft-cr"><circle cx="54" cy="50" r="32"/></clipPath><mask id="ft-lmr"><circle cx="44" cy="50" r="32" fill="#fff"/><circle cx="54" cy="50" r="32" fill="#000"/></mask></defs><g fill="#3fb85a"><g mask="url(#ft-lmr)"><rect x="0" y="13.5" width="100" height="4.9"/><rect x="0" y="22.5" width="100" height="4.9"/><rect x="0" y="31.5" width="100" height="4.9"/><rect x="0" y="40.5" width="100" height="4.9"/><rect x="0" y="49.5" width="100" height="4.9"/><rect x="0" y="58.5" width="100" height="4.9"/><rect x="0" y="67.5" width="100" height="4.9"/><rect x="0" y="76.5" width="100" height="4.9"/><rect x="0" y="85.5" width="100" height="4.9"/></g><g clip-path="url(#ft-cr)"><rect x="0" y="9" width="100" height="4.9"/><rect x="0" y="18" width="100" height="4.9"/><rect x="0" y="27" width="100" height="4.9"/><rect x="0" y="36" width="100" height="4.9"/><rect x="0" y="45" width="100" height="4.9"/><rect x="0" y="54" width="100" height="4.9"/><rect x="0" y="63" width="100" height="4.9"/><rect x="0" y="72" width="100" height="4.9"/><rect x="0" y="81" width="100" height="4.9"/><rect x="0" y="90" width="100" height="4.9"/></g></g></svg>
      IRL · MacroPulse · irl.macropulse.live
    </div>
    <div class="footer-links">
      <a href="https://macropulse.live/irl">Overview</a>
      <a href="https://macropulse.live/irl-whitepaper">Whitepaper</a>
      <a href="https://github.com/macropulse-lab/irl-public-docs">Docs</a>
      <a href="https://macropulse.live/proof">Verify</a>
      <a href="mailto:licensing@macropulse.live">Contact</a>
    </div>
  </div>
</footer>

</body>
</html>
"##;

/// GET /irl/trace/:trace_id
///
/// Returns the full Reasoning_Trace_v1 JSON for forensic audit replay.
/// Overlays live binding fields (final_proof, verification_status) on the stored trace.
/// Decrypts encrypted rows (encryption_version=1) transparently.
pub async fn get_trace(
    State(state): State<AppState>,
    Path(trace_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let pool = state.readonly_pool.as_ref().unwrap_or(&state.pool);
    let trace = db::get_trace_json(pool, trace_id, state.key_provider.as_deref()).await?;
    Ok(Json(trace))
}

#[derive(Deserialize)]
pub struct PendingQuery {
    /// Minimum age in seconds before a trace appears in the pending list.
    /// Default: 0 (show all PENDING traces).
    pub age_seconds: Option<i64>,
}

/// GET /irl/pending
///
/// Returns PENDING traces older than `age_seconds` (default: all PENDING).
/// Used by operators to identify unconfirmed intents awaiting bind-execution.
/// Decrypts encrypted rows (encryption_version=1) transparently.
pub async fn get_pending(
    State(state): State<AppState>,
    Query(q): Query<PendingQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let age = q.age_seconds.unwrap_or(0);
    let pool = state.readonly_pool.as_ref().unwrap_or(&state.pool);
    let traces = db::get_pending_traces(pool, age, state.key_provider.as_deref()).await?;
    Ok(Json(
        serde_json::json!({ "count": traces.len(), "traces": traces }),
    ))
}

/// GET /irl/orphans
///
/// Returns EXPIRED and DIVERGENT traces — trades that were either never confirmed
/// or where the exchange execution differed from the authorized intent.
/// Decrypts encrypted rows (encryption_version=1) transparently.
pub async fn get_orphans(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    let pool = state.readonly_pool.as_ref().unwrap_or(&state.pool);
    let traces = db::get_orphan_traces(pool, state.key_provider.as_deref()).await?;
    Ok(Json(
        serde_json::json!({ "count": traces.len(), "traces": traces }),
    ))
}

/// GET /irl/shadow-violations
///
/// Returns traces where SHADOW_MODE intercepted a policy violation.
/// Used by compliance teams to tune policies before switching to enforcement.
/// Only populated when `SHADOW_MODE=true` has been active.
/// Decrypts encrypted rows (encryption_version=1) transparently.
pub async fn get_shadow_violations(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    let pool = state.readonly_pool.as_ref().unwrap_or(&state.pool);
    let traces = db::get_shadow_violations(pool, state.key_provider.as_deref()).await?;
    Ok(Json(
        serde_json::json!({ "count": traces.len(), "traces": traces }),
    ))
}

/// GET /metrics
///
/// Prometheus text exposition format. Unauthenticated by design — restrict at
/// the network layer (firewall / ingress allowlist) in production.
pub async fn metrics_handler() -> impl IntoResponse {
    match metrics::render() {
        Ok(body) => (
            StatusCode::OK,
            [("content-type", "text/plain; version=0.0.4; charset=utf-8")],
            body,
        )
            .into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
    }
}

/// GET /irl/health
///
/// Returns `{"status": "ok"}` with HTTP 200.
/// When MTLS_ENABLED=true, also includes `cert_expiry_status`.
/// Intended for load-balancer and container health probes.
pub async fn health(State(state): State<AppState>) -> Json<serde_json::Value> {
    use crate::tls::expiry::{check_cert_expiry, CertExpiryStatus};

    if state.config.mtls_enabled {
        if let Some(not_after) = state.cert_expiry_not_after {
            let expiry_status = match check_cert_expiry(not_after) {
                CertExpiryStatus::Ok => serde_json::json!("ok"),
                CertExpiryStatus::ExpiringSoon { days_remaining } => {
                    serde_json::json!({ "warning": format!("expires in {days_remaining} day(s)") })
                }
                CertExpiryStatus::Expired => serde_json::json!("expired"),
            };
            return Json(serde_json::json!({
                "status": "ok",
                "cert_expiry_status": expiry_status
            }));
        }
    }
    Json(serde_json::json!({ "status": "ok" }))
}
