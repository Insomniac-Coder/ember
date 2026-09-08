# Ember — language evolution

A repeatable process for taking the specification from one revision to the next
without losing what earlier revisions already settled.

## Why this exists

The specification is reviewed often and is unusually careful about its own
trade-offs: Part 0 records seventeen rejected alternatives *with reasons*,
`docs/DECISIONS.md` records nine owner-confirmed ADRs, `docs/spec-errata.md`
records seven places the document contradicts itself. A review that does not
read those first produces proposals that were already considered and dismissed,
which costs the owner attention and teaches nothing.

It also costs something worse. v0.3 was authored from v0.2 and did not carry the
four errata rulings the owner had already made, so it silently reverted them and
the document forked from the compiler that implements it. Any process that
evolves this specification has to reconcile before it improves.

## The workflow

`.claude/workflows/ember-evolve.js`, run with the Claude Code `Workflow` tool.
Ten agents in three phases:

**Analyse** — seven independent lenses, in parallel, each reading the full
specification, the ADRs, the errata, the handoff notes and the compiler source:

| Lens | Owns |
|---|---|
| `RECON` | drift between the document, the split spec, and the shipped compiler |
| `ERGO` | the "less annoying than Rust" claim, tested against real code shapes |
| `PYTH` | Python readability and syntax coherence |
| `FFI` | C/C++ interoperability, code reuse, and the adoption ramp for an existing C++ codebase |
| `SOUND` | soundness of the safety model, attacked at the seams |
| `PERF` | whether "speed of C" is a claim the design can support |
| `LOVE` | tooling, diagnostics, the first hour, and what makes a language loved |

Each returns at most ten findings. Every finding must carry a `prior_art_check`
naming the Part 0 row, non-goal, ADR or errata entry it is adjacent to, and why
it is not merely re-proposing something already rejected.

**Challenge** — two adversaries read the complete finding set (they need it whole
to catch duplication across lenses and conflicts between proposals):

- *prior-art* kills anything that re-proposes a rejected alternative without
  defeating its stated reason, contradicts an ADR without saying it is reopening
  one, or is generic commentary that could have been written without reading
  this specification.
- *soundness* attacks the proposals themselves: does the rule interact badly with
  a distant guarantee, is the delta precise enough to implement, is it buildable
  in a Rust compiler emitting C11 with no LLVM until v2, does it conflict with
  another accepted proposal.

A finding survives only if neither challenger rejects it.

**Synthesise** — one editor turns the survivors into a numbered RFC change-set
ordered by value to the vision, each RFC carrying exact normative spec text, cost
and blast radius, and provenance. Rejected findings are listed with their reasons
so the same ground is not covered twice.

## Running it

```
Workflow({ name: 'ember-evolve' })                                  # 0.3 -> 0.4
Workflow({ name: 'ember-evolve', args: { from: '0.4', to: '0.5' } }) # next time
```

Accepted `args`: `root`, `from`, `to`, `spec`, `out`, `notes`.

`notes` is version-specific ground truth injected into every agent's prompt — for
0.3 it is the fact that the four errata rulings were reverted. **Replace it on
each revision.** A stale note is worse than none, because seven agents will treat
it as fact.

The workflow expects the single-file specification at
`docs/spec-source/ember-<from>.md`. `tools/split_spec.py` regenerates
`docs/spec/` from it once a revision is accepted:

```
python tools/split_spec.py docs/spec-source/ember-0.4.md docs/spec
```

## What the output is for

An RFC change-set is a proposal, not a decision. Each RFC states what it costs
and what it breaks so the owner can accept or reject it individually, and the
"Requires an owner decision" section exists because Part XX.1 ground rule 3
forbids an implementing agent from silently choosing an answer to a question the
specification reserves to the owner. Accepted RFCs are applied to the source
document by hand, an ADR is written for anything that alters a confirmed
decision, and `docs/spec/` is regenerated.
