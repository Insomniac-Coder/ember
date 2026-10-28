# 07 word planner: report (run 3, phase 2)

Power: `Online` at the start and at the end.

## What I wrote

| file | what is in it |
|---|---|
| `RUN3/planned-words.md` | the words the five areas start from: the new words (spelling, exact meaning, where written, annoyance ids, why no existing word does, an example with the owner's test applied to every name, what stays refused with its message, what runs); new positions of existing words; the final spelling of every placeholder 06 left open; words considered and not planned; every annoyance AN-01 .. AN-84 with the words its answer starts from; the points each area must rule on |
| `target/redesign3/07-word-planner/p01..p12_*.em`, `check_output.txt` | the probes behind every claim about today's Ember, with the full `ember check` output |

## The decision

1. **Four new words.** `thread_shared` (one of several owners, on any threads; replaces `SyncShared`
   and Rust's `Arc`), `weak` (a link that does not keep its target alive; replaces `Weak[T]` and
   `upgrade`), `set by` (after a `weak` field's type: the language keeps the link equal to the holder;
   last run's back-link mechanism, measured on par with C, respelled so the word sits on the field),
   `@cached` (a method's result kept and cleared when a field its `reads` clause names changes).
   **One argument respelled:** `@fp(allow_fma)` for the placeholder `@fp(fuse)`.
2. **One table for the words that say what a name holds** (`planned-words.md` section 2):
   `copied`, `owned`, `shared`, `thread_shared`, `weak`, each with one meaning before a value, in a type
   and in a capture list, so the five areas cannot spell the same thing three ways.
3. **No new word where an existing one says it:** `ForeignBox[T]` becomes `owned T` (06's `Foreign[T]`
   dropped); module variables use `static mut` with row 3's `changes FRAME` on the function (no
   `global`); `from` also goes after a field's type for self-referential data; `get_or` and `retain`
   already exist for AN-12 and "delete after the loop".
4. **Not planned:** `global`, `isinstance`, `copy(x)`, `backlink(...)` as spelled, `Foreign[T]`, `get`
   for `upgrade`, `shared mut`, a second `unsafe`, `no_share`, `nonlocal`, `lazy` (each with its reason,
   section 6).

The agreed rows 3-9 are unchanged; rows 3, 6 and 7 gain uses (`changes` on a free function, more value
words after `with`, `@cached` built on `reads`) without any change to their spelling or meaning.

## Points the owner may want to see first

- `weak` before a value: `cam.target = weak player`. Without the word, `=` would hand `player` over
  (row 9), so the word goes on the assignment, as Python's `weakref.ref(player)` does.
- `set by`: `parent: weak Node set by children`. Two words; the alternatives tried (`in`, `from`,
  last run's `= backlink(children)`) read wrongly where they sit (section 3.3 says why each).
- `thread_shared` makes the cost of sharing between threads visible in the line; `shared` stays
  one-thread and cheap.
