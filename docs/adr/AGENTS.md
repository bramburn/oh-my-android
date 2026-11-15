# AGENTS.md — docs/adr (scope: docs/adr/)

Architecture Decision Records for this fork. Read the root `AGENTS.md` first.

## Rules

1. One decision per file: `ADR-NNNN-short-slug.md`, zero-padded, never renumbered.
2. Start every new ADR from `ADR-0000-template.md`; status begins as **Proposed**.
3. An ADR is **immutable once Accepted**. To change a decision, write a new ADR that supersedes it and set
   the old one's status to `Superseded by ADR-NNNN`. Do not rewrite history.
4. Required content: context with citations, the decision in the active voice, consequences split into
   positive/negative/neutral, and an alternatives table with *why not*.
5. No ADR without evidence: `path:line` for upstream claims, URL for external API claims, spike numbers for
   performance claims. Mark anything unexecuted `UNVERIFIED`.
6. ADR-0001 (Tauri) and ADR-0003 (repo strategy) are user decisions; ADR-0002 (mirroring) is still open.
   Do not flip ADR-0002 to Accepted without the product owner's explicit answer.

## Index

| ADR | Status | Decision |
|---|---|---|
| 0001 | Accepted | Windows GUI in Tauri v2 (Rust + WebView2 + Svelte 5 TS) |
| 0002 | **Proposed** | Mirroring: spike-gated, A→B→C→D fallback ladder, D always ships |
| 0003 | Accepted | Add the Windows app beside `Sources/**`; never modify or delete it |