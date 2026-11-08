# K42-split-mutspan-halves

Question: How does a program write through both halves of a split MutSpan?
Severity: medium; impact rank 42
Members: J4-09; primary J4-09
Why together: Single question.
Critic's notes: See the recommendation change. Show `if pair exists as (ref mut a, ref b):` (the spelling for get_pair_mut's result, spec 841) beside the options.

## Finding J4-09
Rules: SPN-5, SPN-3, BRW-12, GRM-5; file tasks/spec-0.9.9/parts/p07-ownership.md lines 820-836; kind undecided_case; severity medium
Quote from Hardened_11:
`split_at(i)` on a `Span` returns two `Span`s; on
  a `MutSpan` it lends the view and returns two disjoint `MutSpan`s. `i > len` panics.

The finder's question:
How does a program write through both halves of `split_at` on a `MutSpan`? Does `left, right =
samples.split_at(mid)` followed by `right[i] = ...` compile, and if not, what line does?

Program:
```
fn crossfade(ref mut samples: MutSpan[float]):
    mid = samples.len() // 2
    left, right = samples.split_at(mid)
    for i in 0..mid:
        right[i] = (left[i] + right[i]) / 2.0   # `right` is a plain name holding a MutSpan: E3023 ([SPN-3])?
```

The checker's verdict: confirmed (route owner_odr, severity medium, confidence 0.6)
Reason:
Not settled. [SPN-5] promises two disjoint `MutSpan`s that can be written, but a name that writes a `MutSpan`
must say `ref mut` ([SPN-3]; verdict 34). The assignment grammar takes only names (p03 205-208), and
[BRW-12]'s `ref mut` before a call covers "a `MutSpan` a call gives", not a pair of them. The only spec
example of `split_at` is a read-only `Span`. A one-branch `match` with `(ref mut left, ref mut right)` would
need `ref mut` names bound into a temporary, which [EXP-5] ("a `ref mut` name ... require[s] a place") leaves
doubtful. Verdict 34 decides that such a name says `ref mut`, but not where the word goes on a destructuring
line.

options:
[{'letter': 'A', 'summary': 'The word goes before the value, as on every `=` line: `ref mut` before the call
gives both names the right to write their halves.', 'code': 'left, right = ref mut samples.split_at(mid)',
'outcome': 'Compiles; prints [1.0, 3.0, 3.0, 5.0]. Without `ref mut`, `right[i] = ...` stays E3023, and the
help gives this line.'}, {'letter': 'B', 'summary': "Words go before each name, as a loop's and a pattern's
names take them.", 'code': 'ref mut left, ref mut right = samples.split_at(mid)', 'outcome': 'Compiles; prints
the same. `left, ref mut right = ...` is E0116 under the comma rule; `ref left, ref mut right = ...` writes
only the second half.'}, {'letter': 'C', 'summary': 'No new form: open the pair with a one-branch `match`,
whose pattern names already take words.', 'code': 'match samples.split_at(mid):\n    (ref mut left, ref mut
right):\n        for i in 0..mid:\n            right[i] = (left[i] + right[i]) / 2.0', 'outcome': "Compiles;
prints the same. The spec must then say that a `match` on a call's result may give `ref mut` names to the
spans in it, and every split costs a nested block."}]

recommendation:
{'letter': 'A', 'reason': 'An `=` line already puts the word before the value (`d = ref mut
tracks[dst_name].as_mut_span()`), and `ref mut` before a span already names its elements. Extending that to
the spans a call gives adds the one missing form, and adds no second place for words on an assignment.',
'checklist': "5, 11 (the language's own spelling for an `=` line), 12 (adds the missing syntax instead of
leaving the split unusable), 13"}

