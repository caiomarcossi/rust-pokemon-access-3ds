# pa3ds documentation

Six places, one question each, and a tour.

| Where | Answers | Binding? |
|---|---|---|
| **[`../CONTEXT.md`](../CONTEXT.md)** | *What do we call things?* — the vocabulary the code and the conversation both use. **Start here.** | Vocabulary is |
| **[`adr/`](adr/README.md)** | *Why is it built this way?* — one decision a record, with what it costs and what it rejected. | **Yes** |
| **[`findings/`](findings/README.md)** | *How do we know?* — 79 spikes against the running game, each with what it measured. | Evidence is |
| **[`plans/`](plans/README.md)** | *What are we going to do?* — designed but unbuilt work. | No |
| **[`runbooks/`](runbooks/README.md)** | *How do I do this thing?* — driving the rig, capturing memory, reading the executable. | No |
| **[`reviews/`](reviews/)** | *What did looking at something else teach us?* — dated, about tools we did not build. | No |
| **[`roadmap.md`](roadmap.md)** | *What is left, and what do we still not know?* | No |
| **[`how-pa3ds-works.md`](how-pa3ds-works.md)** | *How does any of this work?* — a tour from the game's memory to the voice, for a coder who is not a reverse engineer, with a glossary. | No |

`agents/` is configuration for the engineering skills — which issue tracker, which triage labels,
where the domain docs are. It is setup, not knowledge.

## Which one do I write?

- A **decision** that would otherwise be re-argued, or that someone could plausibly "clean up"
  without knowing why → an **ADR**. Especially when you *rejected* something: the rejection is the
  valuable half.
- **Something you measured against the game** → a **finding**. Anything a reader believes about
  ORAS should be traceable to one.
- **Designed work you are not building yet** → a **plan**. Delete it when it lands and fold the
  decision into an ADR.
- **A word the code uses inconsistently, or a concept a newcomer would trip on** → **CONTEXT.md**.
- **A procedure you cannot re-derive from the code**, because it lives in an emulator's UI or on
  another machine → a **runbook**.
- **What you just changed** → nothing. The commit and the code say it better.

## Finding things

1. Skim **CONTEXT.md** for the vocabulary of the area.
2. Read the **ADRs** for it — the [index](adr/README.md) is grouped by area.
3. When you need to know *how* something was established rather than what was decided, read the
   **finding**. The numbers in code comments (`spike 73`) are entries in
   [findings/](findings/README.md).
4. Check **plans/** before designing something that may already be designed.

If your work contradicts an ADR, **say so explicitly** rather than silently overriding it.

## History

Until 2026-09-21 all of this was one file, `docs/design.md`, which had grown to 2,822 lines: the
goal, every decision, 79 spike write-ups, the roadmap and the open questions, in one place. It was
split into the above; nothing was rewritten in the move, and git history has the original. The
glossary moved from `docs/domain-model.md` to `CONTEXT.md`, and the agent guides from
`docs/agents/` to `runbooks/`, leaving `agents/` to the skill configuration it was made for.
