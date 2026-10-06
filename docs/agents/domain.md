# Domain Docs

How the engineering skills should consume this repo's domain documentation when exploring the
codebase.

pa3ds is **single-context**, and since 2026-09-21 its documentation is laid out the way the skills
expect:

| The skills expect | pa3ds has |
| ----------------- | --------- |
| `GLOSSARY.md` | [`GLOSSARY.md`](../../GLOSSARY.md) — the glossary, at the repo root |
| `docs/adr/` | [`docs/adr/`](../adr/README.md) — one decision a record, indexed by area |

Beside them, and not something the skills know about: [`docs/findings/`](../findings/README.md),
the evidence — 79 spikes, what each measured and what was built from it.

## Before exploring, read these

- **`GLOSSARY.md`** — the vocabulary. Short; read all of it, and use its words.
- **The ADRs for the area you are about to work in.** The [index](../adr/README.md) groups them.
- **The findings for that area**, when you need to know *how* something was established rather
  than what was decided. `spikes/README.md` maps the scripts to them.

## Where new material goes

| What resolved | Where it goes |
| ------------- | ------------- |
| A term, or a concept the code names inconsistently | `GLOSSARY.md` |
| A decision someone could plausibly undo without knowing why | a new ADR in `docs/adr/` |
| Something measured against the running game | a spike entry in `docs/findings/` |
| Designed work not yet built | `docs/plans/` |
| A procedure that cannot be re-derived from the code | `docs/runbooks/` |
| What you just changed | nothing — the commit and the code say it better |

## Use the glossary's vocabulary

When your output names a domain concept (in an issue title, a refactor proposal, a hypothesis, a
test name), use the term as defined in `GLOSSARY.md`. Don't drift to synonyms the glossary
explicitly avoids.

If the concept you need isn't in the glossary yet, that's a signal — either you're inventing
language the project doesn't use (reconsider) or there's a real gap (note it for
`/domain-modeling`).

## Flag decision conflicts

An ADR is binding. If your output contradicts one, surface it rather than quietly doing otherwise:

> _Contradicts ADR-0010 (nothing interrupts by default) — but worth reopening because…_

Four of them are hard constraints Orion has asked to be consulted about before reopening:
announce-never-act, no hardcoded game text, anchors-not-addresses, and clean-room. `AGENTS.md`
names them.
