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
<link href="https://fonts.googleapis.com/css2?family=Inter:wght@300;400;500;600;700;800;900&family=JetBrains+Mono:wght@400;500;600&family=Plus+Jakarta+Sans:wght@600;700;800&family=Instrument+Serif:ital@1&display=swap" rel="stylesheet">
<style>
  *, *::before, *::after { box-sizing: border-box; margin: 0; padding: 0; }
  :root {
    --bg: #08080a; --s1: #0e0e11; --s2: #141418;
    --glass: rgba(255,255,255,0.025); --glass2: rgba(255,255,255,0.045);
    --line: rgba(255,255,255,0.07); --line2: rgba(255,255,255,0.11);
    --text: #f3f3f5; --muted: #9a9aa2; --dim: #5a5a63;
    --amber: #f5a623; --amber-b: #ffc45e; --amber-dim: rgba(245,166,35,0.1); --amber-brd: rgba(245,166,35,0.3);
    --green: #3fb85a; --green-dim: rgba(63,184,90,0.09);
    --rose: #f97066;
    --blue: #7dd3fc; --grn: #86efac; --str: #fca5a5; --pur: #c4b5fd;
    --sans: 'Inter', -apple-system, sans-serif; --mono: 'JetBrains Mono', monospace;
    --jakarta: 'Plus Jakarta Sans', sans-serif; --serif: 'Instrument Serif', serif;
    --ease: cubic-bezier(0.16, 1, 0.3, 1);
  }
  html { scroll-behavior: smooth; }
  body { background: var(--bg); color: var(--text); font-family: var(--sans); line-height: 1.6; -webkit-font-smoothing: antialiased; overflow-x: hidden; position: relative; }
  a { color: inherit; text-decoration: none; }
  .mono { font-family: var(--mono); }
  ::selection { background: rgba(245,166,35,0.3); }

  /* Site atmosphere (below the fold) */
  .atmos { position: fixed; inset: 0; z-index: 0; pointer-events: none; overflow: hidden; }
  .orb { position: absolute; border-radius: 50%; filter: blur(60px); opacity: 0.5; }
  .orb-2 { width: 520px; height: 520px; background: radial-gradient(circle, rgba(63,184,90,0.07), transparent 62%); top: 1600px; left: -220px; }
  .grain { position: absolute; inset: 0; opacity: 0.035; background-image: url("data:image/svg+xml,%3Csvg viewBox='0 0 200 200' xmlns='http://www.w3.org/2000/svg'%3E%3Cfilter id='n'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='0.9' numOctaves='3'/%3E%3C/filter%3E%3Crect width='100%25' height='100%25' filter='url(%23n)'/%3E%3C/svg%3E"); }
  body > *:not(.atmos) { position: relative; z-index: 1; }

  /* Nav */
  nav { position: fixed; top: 0; left: 0; right: 0; z-index: 100; height: 64px; display: flex; align-items: center; justify-content: space-between; padding: 0 2.25rem; border-bottom: 1px solid transparent; transition: background .3s, border-color .3s; }
  nav.scrolled { background: rgba(8,8,10,0.72); backdrop-filter: blur(18px); border-bottom-color: var(--line); }
  .nav-brand { font-size: 0.98rem; font-weight: 800; letter-spacing: -0.02em; display: flex; align-items: center; gap: 0.55rem; }
  .nav-brand .sub { color: var(--muted); font-weight: 500; }
  .nav-links { display: flex; align-items: center; gap: 2rem; }
  .nav-links a { font-size: 0.86rem; color: rgba(255,255,255,0.72); transition: color 0.2s; }
  .nav-links a:hover { color: var(--amber-b); }
  .nav-cta { font-family: var(--sans); font-weight: 700; font-size: 0.82rem; letter-spacing: 0.03em; color: var(--text) !important; }
  .nav-cta:hover { color: var(--amber-b) !important; }
  @media (max-width: 900px) { .nav-links .hide-sm { display: none; } nav { padding: 0 1.2rem; } }

  .wrap { max-width: 1180px; margin: 0 auto; padding: 0 2.25rem; }
  @media (max-width: 640px) { .wrap { padding: 0 1.25rem; } }

  .btn { display: inline-flex; align-items: center; gap: 0.5rem; border-radius: 100px; font-size: 0.84rem; font-weight: 700; padding: 0.85rem 1.7rem; transition: all 0.18s var(--ease); cursor: pointer; border: none; font-family: var(--sans); text-transform: uppercase; letter-spacing: 0.03em; }
  .btn-primary { background: var(--amber); color: #08080a; box-shadow: 0 10px 34px -12px rgba(245,166,35,0.6); }
  .btn-primary:hover { filter: brightness(1.08); transform: translateY(-2px); box-shadow: 0 16px 44px -14px rgba(245,166,35,0.65); }
  .btn-ghost { border: 1px solid var(--line2); color: var(--muted); background: var(--glass); }
  .btn-ghost:hover { border-color: var(--muted); color: var(--text); }

  /* ===== HERO ===== */
  .hero { position: relative; min-height: 100vh; display: flex; align-items: flex-end; overflow: hidden; }
  .hero-bg { position: absolute; inset: 0; z-index: 0; }
  .streaks { position: absolute; inset: 0; }
  .streaks span { position: absolute; top: -12%; height: 124%; border-radius: 60px; filter: blur(16px); animation: drift 11s ease-in-out infinite; }
  @keyframes drift { 0%,100% { transform: translateY(0); opacity: var(--o,.5); } 50% { transform: translateY(-3%); opacity: calc(var(--o,.5) * 0.7); } }
  .glow-ell { position: absolute; top: -8%; left: 50%; transform: translateX(-50%); width: 72%; height: 380px; background: radial-gradient(ellipse at center, rgba(245,166,35,0.2), rgba(245,166,35,0.05) 45%, transparent 68%); filter: blur(25px); }
  .grid-lines span { position: absolute; top: 0; bottom: 0; width: 1px; background: rgba(255,255,255,0.06); }
  .grid-lines span:nth-child(1) { left: 25%; } .grid-lines span:nth-child(2) { left: 50%; } .grid-lines span:nth-child(3) { left: 75%; }
  @media (max-width: 760px) { .grid-lines { display: none; } }
  .hero-fade { position: absolute; inset: 0; background: linear-gradient(90deg, #08080a 0%, rgba(8,8,10,0.55) 34%, rgba(8,8,10,0) 62%), linear-gradient(0deg, #08080a 4%, rgba(8,8,10,0.2) 34%, transparent 55%); }

  .hero-inner { position: relative; z-index: 2; width: 100%; max-width: 1180px; margin: 0 auto; padding: 120px 2.25rem 4.5rem; min-height: 100vh; display: flex; flex-direction: column; justify-content: flex-end; }
  @media (max-width: 640px) { .hero-inner { padding: 110px 1.25rem 3rem; } }

  /* Liquid glass card */
  .glass-card { position: relative; align-self: flex-start; width: 250px; margin-bottom: 2.5rem; padding: 1.35rem 1.4rem 1.5rem; border-radius: 18px; background: rgba(255,255,255,0.012); background-blend-mode: luminosity; backdrop-filter: blur(5px); -webkit-backdrop-filter: blur(5px); box-shadow: inset 0 1px 1px rgba(255,255,255,0.12), 0 30px 60px -30px rgba(0,0,0,0.7); }
  .glass-card::before { content: ''; position: absolute; inset: 0; border-radius: 18px; padding: 1.4px; background: linear-gradient(180deg, rgba(255,255,255,0.55), rgba(255,255,255,0.04)); -webkit-mask: linear-gradient(#000 0 0) content-box, linear-gradient(#000 0 0); -webkit-mask-composite: xor; mask: linear-gradient(#000 0 0) content-box, linear-gradient(#000 0 0); mask-composite: exclude; pointer-events: none; }
  .glass-card .gc-tag { font-family: var(--mono); font-size: 0.68rem; letter-spacing: 0.14em; color: var(--amber-b); margin-bottom: 0.85rem; }
  .glass-card .gc-h { font-size: 1.18rem; font-weight: 600; letter-spacing: -0.01em; line-height: 1.2; margin-bottom: 0.55rem; }
  .glass-card .gc-h em { font-family: var(--serif); font-style: italic; font-weight: 400; font-size: 1.32rem; }
  .glass-card .gc-d { font-size: 0.72rem; color: rgba(255,255,255,0.55); line-height: 1.5; margin-bottom: 1rem; }
  .glass-card .gc-arrow { width: 30px; height: 30px; border-radius: 50%; border: 1px solid var(--line2); display: flex; align-items: center; justify-content: center; color: var(--muted); transition: all .2s; }
  .glass-card:hover .gc-arrow { border-color: var(--amber-brd); color: var(--amber-b); transform: translateX(2px); }
  @media (max-width: 900px) { .glass-card { position: relative; top: 0; left: 0; margin-bottom: 2.5rem; } }

  .eyebrow-2 { font-family: var(--jakarta); font-weight: 700; font-size: 0.72rem; letter-spacing: 0.16em; text-transform: uppercase; color: var(--amber-b); margin-bottom: 1.3rem; }
  h1.mega { font-family: var(--sans); font-weight: 800; text-transform: uppercase; letter-spacing: -0.035em; line-height: 0.94; font-size: clamp(2.6rem, 8vw, 5rem); margin-bottom: 1.6rem; }
  h1.mega .dot { color: var(--amber); }
  .mega-sub { font-size: clamp(0.95rem, 1.4vw, 1.08rem); color: rgba(255,255,255,0.7); max-width: 512px; line-height: 1.7; margin-bottom: 2.25rem; }
  .hero-actions { display: flex; gap: 0.8rem; flex-wrap: wrap; align-items: center; }
  .trust-strip { display: flex; gap: 1.4rem; flex-wrap: wrap; margin-top: 2.75rem; font-family: var(--mono); font-size: 0.74rem; color: var(--dim); }
  .trust-strip span { display: inline-flex; align-items: center; gap: 0.45rem; }
  .trust-strip .d { color: var(--amber); }

  /* Sections */
  section { padding: 5.5rem 0; border-top: 1px solid var(--line); }
  .reveal { opacity: 0; transform: translateY(22px); transition: opacity .7s var(--ease), transform .7s var(--ease); }
  .reveal.in { opacity: 1; transform: none; }
  .s-label { font-family: var(--mono); font-size: 0.72rem; color: var(--amber-b); letter-spacing: 0.12em; text-transform: uppercase; font-weight: 600; margin-bottom: 0.95rem; }
  h2 { font-size: clamp(1.6rem, 3.2vw, 2.5rem); font-weight: 800; letter-spacing: -0.035em; line-height: 1.1; margin-bottom: 1rem; }
  h2 em { font-family: var(--serif); font-style: italic; font-weight: 400; color: var(--amber-b); }
  .lead { font-size: 1.05rem; color: var(--muted); max-width: 620px; line-height: 1.7; }

  /* Problem */
  .prob-grid { display: grid; grid-template-columns: repeat(3, 1fr); gap: 1.1rem; margin-top: 2.75rem; }
  @media (max-width: 800px) { .prob-grid { grid-template-columns: 1fr; } }
  .prob { background: var(--glass); border: 1px solid var(--line); border-radius: 14px; padding: 1.7rem; transition: border-color .25s, transform .25s var(--ease), background .25s; }
  .prob:hover { border-color: rgba(249,112,102,0.35); transform: translateY(-3px); background: var(--glass2); }
  .prob .n { font-family: var(--mono); font-size: 0.68rem; color: var(--rose); margin-bottom: 0.85rem; letter-spacing: 0.08em; }
  .prob h3 { font-size: 1.05rem; font-weight: 700; margin-bottom: 0.5rem; letter-spacing: -0.01em; }
  .prob p { font-size: 0.88rem; color: var(--muted); line-height: 1.6; }

  /* Flow */
  .flow { display: grid; grid-template-columns: repeat(3, 1fr); gap: 0; margin-top: 3rem; position: relative; }
  @media (max-width: 800px) { .flow { grid-template-columns: 1fr; gap: 1rem; } }
  .fnode { padding: 2.1rem 1.7rem; border: 1px solid var(--line); background: var(--glass); position: relative; transition: background .25s, border-color .25s; }
  .flow .fnode:first-child { border-radius: 14px 0 0 14px; }
  .flow .fnode:last-child { border-radius: 0 14px 14px 0; }
  .flow .fnode:not(:last-child) { border-right: none; }
  @media (max-width: 800px) { .fnode, .flow .fnode:first-child, .flow .fnode:last-child { border-radius: 14px; border-right: 1px solid var(--line); } }
  .fnode:hover { background: linear-gradient(180deg, var(--amber-dim), transparent); border-color: var(--amber-brd); }
  .fnode .fn-ico { width: 42px; height: 42px; border-radius: 11px; background: var(--amber-dim); border: 1px solid var(--amber-brd); display: flex; align-items: center; justify-content: center; margin-bottom: 1.15rem; color: var(--amber); }
  .fnode .fn-step { font-family: var(--mono); font-size: 0.68rem; color: var(--amber-b); letter-spacing: 0.1em; margin-bottom: 0.5rem; }
  .fnode h3 { font-size: 1.18rem; font-weight: 800; letter-spacing: -0.02em; margin-bottom: 0.55rem; }
  .fnode p { font-size: 0.87rem; color: var(--muted); line-height: 1.62; }
  .fnode code { font-family: var(--mono); font-size: 0.76rem; color: var(--amber-b); background: rgba(245,166,35,0.09); padding: 0.08rem 0.35rem; border-radius: 4px; }
  .fnode .arrow { position: absolute; right: -11px; top: 50%; transform: translateY(-50%); width: 22px; height: 22px; border-radius: 50%; background: var(--s2); border: 1px solid var(--line2); display: flex; align-items: center; justify-content: center; font-size: 0.7rem; color: var(--amber); z-index: 2; }
  @media (max-width: 800px) { .fnode .arrow { display: none; } }

  /* Verify */
  .verify { background: linear-gradient(160deg, var(--s1), rgba(20,16,8,0.4)); border: 1px solid var(--line2); border-radius: 20px; padding: clamp(1.9rem, 4vw, 3.25rem); position: relative; overflow: hidden; }
  .verify::after { content: ''; position: absolute; top: -60%; right: -20%; width: 500px; height: 500px; background: radial-gradient(circle, rgba(245,166,35,0.1), transparent 60%); pointer-events: none; }
  .verify > * { position: relative; }
  .verify .big { font-size: clamp(1.15rem, 2.3vw, 1.6rem); font-weight: 700; letter-spacing: -0.025em; margin: 1.6rem 0 0.4rem; }
  .verify .big .a { color: var(--amber-b); }
  .verify-actions { display: flex; gap: 0.7rem; flex-wrap: wrap; margin-top: 1.9rem; }
  .chip { display: inline-flex; align-items: center; gap: 0.45rem; font-family: var(--mono); font-size: 0.78rem; color: var(--muted); border: 1px solid var(--line2); border-radius: 8px; padding: 0.6rem 1rem; background: var(--glass); transition: all 0.2s var(--ease); }
  .chip:hover { border-color: var(--amber-brd); color: var(--text); transform: translateY(-2px); }

  /* Proof */
  .proof { display: flex; gap: 1.6rem; align-items: flex-start; padding: 2rem; border: 1px solid rgba(63,184,90,0.28); border-radius: 16px; background: var(--green-dim); margin-top: 2.5rem; }
  .proof .mark { flex-shrink: 0; }
  .proof h3 { font-size: 1.1rem; font-weight: 800; margin-bottom: 0.55rem; letter-spacing: -0.01em; }
  .proof p { font-size: 0.92rem; color: var(--muted); line-height: 1.68; }
  .proof a.inline { color: var(--green); border-bottom: 1px solid rgba(63,184,90,0.45); }
  @media (max-width: 560px) { .proof { flex-direction: column; gap: 1rem; } }

  /* Reg */
  .reg-grid { display: grid; grid-template-columns: repeat(4, 1fr); gap: 0.9rem; margin-top: 2.5rem; }
  @media (max-width: 800px) { .reg-grid { grid-template-columns: repeat(2, 1fr); } }
  .reg { border: 1px solid var(--line); border-radius: 12px; padding: 1.3rem 1.4rem; background: var(--glass); transition: border-color .25s, transform .25s; }
  .reg:hover { border-color: var(--amber-brd); transform: translateY(-3px); }
  .reg .name { font-family: var(--mono); font-size: 0.85rem; font-weight: 600; color: var(--text); margin-bottom: 0.35rem; }
  .reg .what { font-size: 0.77rem; color: var(--muted); line-height: 1.5; }

  /* Pricing */
  .price-grid { display: grid; grid-template-columns: repeat(3, 1fr); gap: 1.1rem; margin-top: 2.75rem; align-items: start; }
  @media (max-width: 860px) { .price-grid { grid-template-columns: 1fr; } }
  .tier { border: 1px solid var(--line); border-radius: 16px; padding: 1.9rem; background: var(--glass); display: flex; flex-direction: column; transition: transform .25s var(--ease), border-color .25s; }
  .tier:hover { transform: translateY(-4px); border-color: var(--line2); }
  .tier.featured { border-color: var(--amber-brd); background: linear-gradient(180deg, var(--amber-dim), var(--glass)); box-shadow: 0 0 60px -25px rgba(245,166,35,0.55); }
  .tier .badge { display: inline-block; font-family: var(--mono); font-size: 0.62rem; font-weight: 700; letter-spacing: 0.1em; text-transform: uppercase; color: var(--amber-b); border: 1px solid var(--amber-brd); background: var(--amber-dim); padding: 0.2rem 0.55rem; border-radius: 5px; margin-bottom: 0.95rem; align-self: flex-start; }
  .tier .tname { font-size: 0.95rem; font-weight: 800; margin-bottom: 0.35rem; }
  .tier .tagline { font-size: 0.82rem; color: var(--muted); margin-bottom: 1.1rem; min-height: 2.5em; line-height: 1.45; }
  .tier .price { font-family: var(--mono); font-size: 2.1rem; font-weight: 600; letter-spacing: -0.03em; line-height: 1; }
  .tier .price .per { font-size: 0.78rem; color: var(--muted); font-weight: 400; font-family: var(--sans); }
  .tier .minq { font-family: var(--mono); font-size: 0.66rem; color: var(--dim); text-transform: uppercase; letter-spacing: 0.06em; margin: 0.55rem 0 1.35rem; }
  .tier ul { list-style: none; display: flex; flex-direction: column; gap: 0.6rem; margin-bottom: 1.6rem; flex: 1; }
  .tier li { font-size: 0.84rem; color: var(--muted); display: flex; gap: 0.55rem; align-items: flex-start; line-height: 1.5; }
  .tier li::before { content: '+'; color: var(--amber); font-weight: 700; flex-shrink: 0; }
  .tier li.inherit::before { content: '\21B3'; color: var(--dim); }
  .tier .tcta { text-align: center; padding: 0.72rem; border-radius: 100px; font-size: 0.84rem; font-weight: 700; text-transform: uppercase; letter-spacing: 0.03em; transition: all 0.18s; }
  .tier .tcta.solid { background: var(--amber); color: #08080a; box-shadow: 0 8px 26px -10px rgba(245,166,35,0.55); }
  .tier .tcta.solid:hover { filter: brightness(1.08); transform: translateY(-1px); }
  .tier .tcta.ghost { border: 1px solid var(--line2); color: var(--muted); }
  .tier .tcta.ghost:hover { border-color: var(--muted); color: var(--text); }
  .price-note { text-align: center; font-size: 0.82rem; color: var(--dim); margin-top: 2rem; }
  .price-note a { color: var(--muted); border-bottom: 1px solid var(--line2); }

  /* Sandbox */
  .sandbox { background: var(--s1); border: 1px solid var(--line2); border-radius: 18px; padding: clamp(1.9rem, 4vw, 2.75rem); }
  .sandbox h2 { margin-bottom: 0.4rem; }
  .sandbox pre { background: var(--bg); border: 1px solid var(--line2); border-radius: 10px; padding: 1.15rem 1.25rem; font-family: var(--mono); font-size: 0.78rem; overflow-x: auto; line-height: 1.7; margin-top: 1.6rem; }
  .comment { color: var(--dim); } .key { color: var(--blue); } .val { color: var(--grn); } .str { color: var(--str); } .url { color: var(--pur); }
  .sandbox-links { display: flex; gap: 0.7rem; flex-wrap: wrap; margin-top: 1.6rem; }

  /* Final */
  .final { text-align: center; padding: 6.5rem 0; }
  .final h2 { font-size: clamp(2rem, 5vw, 3.2rem); margin-bottom: 1rem; text-transform: uppercase; letter-spacing: -0.035em; }
  .final h2 .dot { color: var(--amber); }
  .final p { color: var(--muted); max-width: 520px; margin: 0 auto 2.25rem; font-size: 1.05rem; }
  .final .hero-actions { justify-content: center; }

  footer { border-top: 1px solid var(--line); padding: 2.25rem; }
  .footer-in { max-width: 1180px; margin: 0 auto; display: flex; align-items: center; justify-content: space-between; flex-wrap: wrap; gap: 1rem; }
  .footer-brand { font-size: 0.82rem; color: var(--dim); display: flex; align-items: center; gap: 0.55rem; }
  .footer-links { display: flex; gap: 1.4rem; flex-wrap: wrap; }
  .footer-links a { font-size: 0.82rem; color: var(--dim); transition: color 0.15s; }
  .footer-links a:hover { color: var(--muted); }
</style>
</head>
<body>

<div class="atmos">
  <div class="orb orb-2"></div>
  <div class="grain"></div>
</div>

<nav id="nav">
  <a class="nav-brand" href="/">
    <svg width="23" height="23" viewBox="0 0 100 100" fill="none" style="flex-shrink:0" aria-label="MacroPulse"><defs><clipPath id="lg-cr"><circle cx="54" cy="50" r="32"/></clipPath><mask id="lg-lmr"><circle cx="44" cy="50" r="32" fill="#fff"/><circle cx="54" cy="50" r="32" fill="#000"/></mask></defs><g fill="#3fb85a"><g mask="url(#lg-lmr)"><rect x="0" y="13.5" width="100" height="4.9"/><rect x="0" y="22.5" width="100" height="4.9"/><rect x="0" y="31.5" width="100" height="4.9"/><rect x="0" y="40.5" width="100" height="4.9"/><rect x="0" y="49.5" width="100" height="4.9"/><rect x="0" y="58.5" width="100" height="4.9"/><rect x="0" y="67.5" width="100" height="4.9"/><rect x="0" y="76.5" width="100" height="4.9"/><rect x="0" y="85.5" width="100" height="4.9"/></g><g clip-path="url(#lg-cr)"><rect x="0" y="9" width="100" height="4.9"/><rect x="0" y="18" width="100" height="4.9"/><rect x="0" y="27" width="100" height="4.9"/><rect x="0" y="36" width="100" height="4.9"/><rect x="0" y="45" width="100" height="4.9"/><rect x="0" y="54" width="100" height="4.9"/><rect x="0" y="63" width="100" height="4.9"/><rect x="0" y="72" width="100" height="4.9"/><rect x="0" y="81" width="100" height="4.9"/><rect x="0" y="90" width="100" height="4.9"/></g></g></svg>
    IRL <span class="sub">· MacroPulse</span>
  </a>
  <div class="nav-links">
    <a href="#how" class="hide-sm">How it works</a>
    <a href="#verify" class="hide-sm">Verify</a>
    <a href="#pricing" class="hide-sm">Pricing</a>
    <a href="#sandbox" class="hide-sm">Sandbox</a>
    <a href="https://github.com/macropulse-lab/irl-public-docs" class="hide-sm">Docs</a>
    <a href="#pricing" class="nav-cta">[ Get access ]</a>
  </div>
</nav>

<header class="hero">
  <div class="hero-bg">
    <div class="streaks">
      <span style="left:52%;width:26px;background:linear-gradient(180deg,transparent,rgba(245,166,35,0.55),transparent);--o:.55;animation-delay:0s"></span>
      <span style="left:58%;width:14px;background:linear-gradient(180deg,transparent,rgba(255,196,94,0.5),transparent);--o:.5;animation-delay:1.2s"></span>
      <span style="left:64%;width:40px;background:linear-gradient(180deg,transparent,rgba(245,166,35,0.32),transparent);--o:.45;animation-delay:2.4s"></span>
      <span style="left:71%;width:18px;background:linear-gradient(180deg,transparent,rgba(255,255,255,0.14),transparent);--o:.4;animation-delay:0.6s"></span>
      <span style="left:78%;width:30px;background:linear-gradient(180deg,transparent,rgba(180,110,20,0.5),transparent);--o:.5;animation-delay:1.8s"></span>
      <span style="left:85%;width:16px;background:linear-gradient(180deg,transparent,rgba(255,196,94,0.4),transparent);--o:.4;animation-delay:3s"></span>
      <span style="left:91%;width:34px;background:linear-gradient(180deg,transparent,rgba(245,166,35,0.28),transparent);--o:.4;animation-delay:0.9s"></span>
      <span style="left:46%;width:12px;background:linear-gradient(180deg,transparent,rgba(255,255,255,0.1),transparent);--o:.35;animation-delay:2.1s"></span>
    </div>
    <div class="glow-ell"></div>
    <div class="grid-lines"><span></span><span></span><span></span></div>
    <div class="hero-fade"></div>
  </div>

  <div class="hero-inner">
    <a href="https://macropulse.live/proof" class="glass-card">
      <div class="gc-tag">[ VERIFY ]</div>
      <div class="gc-h">Check it <em>yourself.</em></div>
      <div class="gc-d">Open-source verifier. Offline. Against Bitcoin — no account, no trust in us.</div>
      <div class="gc-arrow">→</div>
    </a>

    <div class="eyebrow-2">Pre-execution compliance for AI agents</div>
    <h1 class="mega">Prove what your<br>agent decided<span class="dot">.</span></h1>
    <p class="mega-sub">IRL seals your agent's reasoning the instant before an order reaches the exchange, binds it to the fill, and anchors it to Bitcoin. A tamper-evident record of every decision.</p>
    <div class="hero-actions">
      <a href="#sandbox" class="btn btn-primary">Try the sandbox →</a>
      <a href="https://macropulse.live/irl-whitepaper" class="btn btn-ghost">Whitepaper</a>
    </div>
    <div class="trust-strip">
      <span><span class="d">◆</span> SHA-256 sealed</span>
      <span><span class="d">◆</span> Bitcoin anchored</span>
      <span><span class="d">◆</span> Verify offline</span>
    </div>
  </div>
</header>

<!-- Problem -->
<section id="problem">
  <div class="wrap reveal">
    <div class="s-label">The problem</div>
    <h2>An AI agent just made a trade you <em>can't explain.</em></h2>
    <p class="lead">Autonomous agents decide in milliseconds, faster than any human reviews. When one loses big — or does something that looks like abuse — the only record is a log file you control. That's not a defense.</p>
    <div class="prob-grid">
      <div class="prob"><div class="n">01 · UNEXPLAINABLE</div><h3>No reasoning trail</h3><p>The model acted on state that's already gone. Reconstructing "why" after the fact is guesswork a regulator won't accept.</p></div>
      <div class="prob"><div class="n">02 · EDITABLE</div><h3>Logs can be rewritten</h3><p>Records that live on your own servers can be changed — so they prove nothing to an auditor, an LP, or a court.</p></div>
      <div class="prob"><div class="n">03 · LIABILITY</div><h3>"Trust us" is the exposure</h3><p>With no independent proof, every automated decision is a liability waiting for a subpoena you can't answer.</p></div>
    </div>
  </div>
</section>

<!-- How -->
<section id="how">
  <div class="wrap reveal">
    <div class="s-label">How it works</div>
    <h2>Seal. Bind. Anchor.</h2>
    <p class="lead">IRL runs as a gateway between your agent and the exchange. Three steps turn an automated decision into permanent, checkable evidence.</p>
    <div class="flow">
      <div class="fnode">
        <div class="fn-ico"><svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8"><rect x="5" y="11" width="14" height="9" rx="2"/><path d="M8 11V7a4 4 0 0 1 8 0v4"/></svg></div>
        <div class="fn-step">01 — SEAL</div>
        <h3>Before the order</h3>
        <p>Your agent submits its reasoning — model, intent, size, risk checks. IRL runs pre-execution policy, then seals the snapshot with <code>SHA-256</code>. Nothing trades until it passes.</p>
        <div class="arrow">→</div>
      </div>
      <div class="fnode">
        <div class="fn-ico"><svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8"><path d="M9 12a3 3 0 0 1 3-3h1a3 3 0 0 1 0 6h-1"/><path d="M15 12a3 3 0 0 1-3 3h-1a3 3 0 0 1 0-6h1"/></svg></div>
        <div class="fn-step">02 — BIND</div>
        <h3>After the fill</h3>
        <p>The exchange returns a transaction ID. IRL computes <code>final_proof</code> from the sealed reasoning and the tx — binding intent to what actually executed.</p>
        <div class="arrow">→</div>
      </div>
      <div class="fnode">
        <div class="fn-ico"><svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8"><path d="M12 2v4M12 18v4M2 12h4M18 12h4"/><circle cx="12" cy="12" r="4"/></svg></div>
        <div class="fn-step">03 — ANCHOR</div>
        <h3>Every day</h3>
        <p>All seals roll into a daily Merkle root, anchored to Bitcoin via OpenTimestamps. The record becomes tamper-evident against everyone — including us.</p>
      </div>
    </div>
  </div>
</section>

<!-- Verify -->
<section id="verify">
  <div class="wrap reveal">
    <div class="verify">
      <div class="s-label">Trust without trust</div>
      <h2>You don't have to <em>trust us.</em></h2>
      <p class="lead">Every audit vendor claims a tamper-proof log. Only IRL hands the proof to the other side — the auditor, the regulator, the LP — and lets them check it with zero access to our servers.</p>
      <div class="big"><span class="a">Open-source verifier.</span> Frozen spec. Reimplement it in any language.</div>
      <div class="big">Check any proof bundle <span class="a">offline, against Bitcoin.</span></div>
      <p class="lead" style="margin-top:1.1rem;">We can't fake it. We can't rewrite it. Neither can you. That's the entire design.</p>
      <div class="verify-actions">
        <a href="https://github.com/macropulse-lab/irl-verify" class="chip">↗ irl-verify (MIT, offline)</a>
        <a href="https://macropulse.live/proof" class="chip">↗ In-browser proof explorer</a>
        <a href="https://github.com/macropulse-lab/irl-public-docs" class="chip">↗ Frozen spec</a>
      </div>
    </div>
  </div>
</section>

<!-- Proof -->
<section id="proof">
  <div class="wrap reveal">
    <div class="s-label">Why us</div>
    <h2>We run our own signal on this discipline.</h2>
    <div class="proof">
      <svg class="mark" width="58" height="58" viewBox="0 0 100 100" fill="none" aria-label="MacroPulse"><defs><clipPath id="pf-cr"><circle cx="54" cy="50" r="32"/></clipPath><mask id="pf-lmr"><circle cx="44" cy="50" r="32" fill="#fff"/><circle cx="54" cy="50" r="32" fill="#000"/></mask></defs><g fill="#3fb85a"><g mask="url(#pf-lmr)"><rect x="0" y="13.5" width="100" height="4.9"/><rect x="0" y="22.5" width="100" height="4.9"/><rect x="0" y="31.5" width="100" height="4.9"/><rect x="0" y="40.5" width="100" height="4.9"/><rect x="0" y="49.5" width="100" height="4.9"/><rect x="0" y="58.5" width="100" height="4.9"/><rect x="0" y="67.5" width="100" height="4.9"/><rect x="0" y="76.5" width="100" height="4.9"/><rect x="0" y="85.5" width="100" height="4.9"/></g><g clip-path="url(#pf-cr)"><rect x="0" y="9" width="100" height="4.9"/><rect x="0" y="18" width="100" height="4.9"/><rect x="0" y="27" width="100" height="4.9"/><rect x="0" y="36" width="100" height="4.9"/><rect x="0" y="45" width="100" height="4.9"/><rect x="0" y="54" width="100" height="4.9"/><rect x="0" y="63" width="100" height="4.9"/><rect x="0" y="72" width="100" height="4.9"/><rect x="0" y="81" width="100" height="4.9"/><rect x="0" y="90" width="100" height="4.9"/></g></g></svg>
      <div>
        <h3>IRL is built by the team behind MacroPulse.</h3>
        <p>MacroPulse is a macro-regime signal we run in production — Ed25519-signed and publicly verifiable, every day. Proving our own market calls is how we learned how hard real proof is. IRL's Layer 2 can bind each decision to that signed market truth. <a class="inline" href="https://macropulse.live/track-record">See the public track record →</a></p>
      </div>
    </div>
  </div>
</section>

<!-- Reg -->
<section id="reg">
  <div class="wrap reveal">
    <div class="s-label">Built for what's coming</div>
    <h2>Auditability is becoming <em>law.</em></h2>
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
  <div class="wrap reveal">
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
  <div class="wrap reveal">
    <div class="sandbox">
      <div class="s-label" style="margin-bottom:0.6rem;">For developers</div>
      <h2>Try it now. <em>No signup.</em></h2>
      <p class="lead">Three demo agents are pre-seeded. Run a full authorize → bind flow in the interactive API, then verify the proof bundle yourself.</p>
      <pre><span class="comment"># POST /irl/authorize — seal the reasoning before the order</span>
{
  <span class="key">"agent_id"</span>: <span class="str">"00000000-0000-4000-a000-000000000001"</span>,
  <span class="key">"model_hash"</span>: <span class="str">"sha256-hex-of-model"</span>,
  <span class="key">"action"</span>: { <span class="key">"Long"</span>: <span class="val">1.5</span> },
  <span class="key">"asset"</span>: <span class="str">"BTC/USD"</span>,
  <span class="key">"notional"</span>: <span class="val">64875.00</span>
}
<span class="comment"># → { "trace_id": "…", "reasoning_hash": "sha256…", "authorized": true }</span></pre>
      <div class="sandbox-links">
        <a href="/swagger-ui/" class="btn btn-primary">Open interactive API →</a>
        <a href="/openapi.json" class="btn btn-ghost">OpenAPI JSON</a>
        <a href="/health" class="btn btn-ghost">Health</a>
      </div>
    </div>
  </div>
</section>

<!-- Final -->
<div class="wrap">
  <div class="final reveal">
    <h2>Every agent decision, provable<span class="dot">.</span></h2>
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
      <svg width="17" height="17" viewBox="0 0 100 100" fill="none"><defs><clipPath id="ft-cr"><circle cx="54" cy="50" r="32"/></clipPath><mask id="ft-lmr"><circle cx="44" cy="50" r="32" fill="#fff"/><circle cx="54" cy="50" r="32" fill="#000"/></mask></defs><g fill="#3fb85a"><g mask="url(#ft-lmr)"><rect x="0" y="13.5" width="100" height="4.9"/><rect x="0" y="22.5" width="100" height="4.9"/><rect x="0" y="31.5" width="100" height="4.9"/><rect x="0" y="40.5" width="100" height="4.9"/><rect x="0" y="49.5" width="100" height="4.9"/><rect x="0" y="58.5" width="100" height="4.9"/><rect x="0" y="67.5" width="100" height="4.9"/><rect x="0" y="76.5" width="100" height="4.9"/><rect x="0" y="85.5" width="100" height="4.9"/></g><g clip-path="url(#ft-cr)"><rect x="0" y="9" width="100" height="4.9"/><rect x="0" y="18" width="100" height="4.9"/><rect x="0" y="27" width="100" height="4.9"/><rect x="0" y="36" width="100" height="4.9"/><rect x="0" y="45" width="100" height="4.9"/><rect x="0" y="54" width="100" height="4.9"/><rect x="0" y="63" width="100" height="4.9"/><rect x="0" y="72" width="100" height="4.9"/><rect x="0" y="81" width="100" height="4.9"/><rect x="0" y="90" width="100" height="4.9"/></g></g></svg>
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

<script>
  var nav = document.getElementById('nav');
  window.addEventListener('scroll', function(){ nav.classList.toggle('scrolled', window.scrollY > 20); }, {passive:true});
  var io = new IntersectionObserver(function(es){ es.forEach(function(e){ if(e.isIntersecting){ e.target.classList.add('in'); io.unobserve(e.target);} }); }, {threshold: 0.12});
  document.querySelectorAll('.reveal').forEach(function(el){ io.observe(el); });
</script>
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
