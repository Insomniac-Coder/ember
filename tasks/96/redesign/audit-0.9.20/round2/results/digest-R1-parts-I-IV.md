# R1-parts-I-IV

could_not_cover: Every cluster was checked against CURRENT-SPEC.md's rule text, the grammar and the registry rows; Appendix H §H.52/§H.53 were not read line by line (the changed rules themselves were read instead), and no compiler source was consulted, so where a reading rests on an omission (K45's `owns dyn I` having no `Ord` row; K70's raw identifier) the conclusion is from the text alone.

## K18-string-plus  reader=answered_differently  refuter=narrowed  should_be=answered_differently  safety=none

**answered_by**: TYP-21: "Operators desugar to these interfaces for non-scalar operands, but one: `a + b` calls `Add.add(a, b)` with both operands read in place, and `String`'s `+` is the language's own, taking `a` over (`[TXT-11]`), so `String` implements no operator interface and does not meet `T: needs Add`."; TXT-11: "`a + b` for `a: String, b: StrView` takes `a` over and returns it extended; `s += t` appends."; GRM-47: "An operator reads its operands in place (`a + b` calls `Add.add(a, b)` with both read in place, `[TYP-21]`), so `owns` before an operand hands nothing to the operator and only ends the name ... an `owns` expression that is an operand of an operator of rows 4 to 17 of the table (`or` to `**` ...), with or without brackets of its own, is `E0119`"; EXP-6: "a field of a class object, which other names may reach and would see the hole, and anything reached through a name that does not own it, are `E3013` (`[OWN-11]`)."; FN-2: "A parameter with no word reads."; registry E0119: "`owns` as an operand of an operator (`owns i * 7`), which reads its operands in place and takes nothing over"

**program**: Decided, as the checker's option C: `full = first + " Lovelace"` compiles and hands `first` over with no word on the line (`println(first)` after it is `E3040`); `suffix(owns t, "?")` under `[T: needs Add[StrView, Output = T]]` is `E2040` (TYP-21 now says so in words); `return name + "!"` on a read-only `name: String` parameter is `E3013` (EXP-6: a hand-over through a name that does not own the value). Option B, `(owns label) + "!"`, is foreclosed by GRM-47/N79: `E0119` "with or without brackets of its own".

- READER EDIT rule=TXT-11
  - old: `a + b` for `a: String, b:
  StrView` takes `a` over and returns it extended; `s += t` appends. `StrView + StrView` is `E2040`
  whose help is an f-string.
  - new: `a + b` for `a: String, b: StrView` reads both and gives a new `String`, `a`'s text then `b`'s (`String` implements `Add[StrView, Output = String]`, `[TYP-21]`; one allocation, as an f-string makes), so `a` stays usable; `s += t` appends in place (`AddAssign[StrView]`) and allocates only to grow, which is how text is built in a loop. `StrView + StrView` is `E2040` whose help is an f-string.
  - why: The one operator that takes its left side over does so with no word on the line, which fails the owner's test of 2026-10-24 ("For every name, point at the word in the line that says what it is ... If no word says it, the proposal hides meaning and fails"); a word cannot be added, since GRM-47 (N79) makes `owns` before an operand `E0119`; so the only consistent shape is the one every other operator has, both operands read in place. The cost is one allocation per `+`, which `+=` avoids, the same as C++'s `a + b` on two named strings and as the f-string the help already recommends.

- READER EDIT rule=TYP-21
  - old: Operators desugar to
  these interfaces for non-scalar operands, but one: `a + b` calls `Add.add(a, b)` with both
  operands read in place, and `String`'s `+` is the language's own, taking `a` over (`[TXT-11]`), so
  `String` implements no operator interface and does not meet `T: needs Add`.
  - new: Operators desugar to these interfaces for non-scalar operands: `a + b` calls `Add.add(a, b)` with both operands read in place, for `String` too (`String` implements `Add[StrView, Output = String]` and `AddAssign[StrView]`, `[TXT-11]`, so it meets `T: needs Add[StrView, Output = T]`, and `a + b` on it makes a new `String`).
  - why: The exception goes with TXT-11's change, so the operator rule has no exception and GRM-47's "An operator reads its operands in place" and E0119's "which reads its operands in place and takes nothing over" become true for every operator.

- READER EDIT rule=TYP-21
  - old: `bool`, `char` and `String` implement none of them.
  - new: `bool` and `char` implement none of them.
  - why: `String` now implements `Add` and `AddAssign`.

**remaining_doubt**: The current text is safe (a use after the hand-over is `E3040`, a hand-over through a read-only name `E3013`) and was made explicit in 0.9.20, so the lead may keep C; then the smallest consistency patch is in GRM-47 ("An operator reads its operands in place (but `String`'s `+`, `[TXT-11]`, which takes its left operand over; `owns` before it is `E0119` all the same)") and in the E0119 registry row, whose reason "takes nothing over" is false for `String`, and TXT-11 should name the read-only-name case (`E3013`, help `String(name) + "!"` or `f"{name}!"`). No verdict covers an operator's operand; the owner was never asked, and the two options are the finder's A and C. B is no longer available without excepting `+` from GRM-47.

**REFUTER reason**: The current text does decide the three programs as the reader says: TXT-11 "`a + b` for `a: String, b: StrView` takes `a` over and returns it extended"; TYP-21 "`String`'s `+` is the language's own, taking `a` over (`[TXT-11]`), so `String` implements no operator interface and does not meet `T: needs Add`" (so `suffix(owns t, "?")` under `needs Add` is `E2040`); OWN-3 "Where `a` is itself a view (a plain parameter, ...) ... a view hands nothing over" with the E3013 row "a hand-over out of a name or place that does not own the value: a plain parameter's value" (so `return name + "!"` is `E3013`). But the decision contradicts two rules of the same text: GRM-47 "An operator reads its operands in place (`a + b` calls `Add.add(a, b)` with both read in place, `[TYP-21]`), so `owns` before an operand hands nothing to the operator" and the E0119 row "`owns` as an operand of an operator (`owns i * 7`), which reads its operands in place and takes nothing over", both false for `String`; and EXP-6's list of the uses that hand a whole name over ("given with `=`, given to an `owns` parameter, returned, captured with `owns`") and OWN-9's ("`return a` hands `a` over with no word, and so does a constructor's argument, as `=` does") name no operand, so the one word-less hand-over by an operator is stated only in TXT-11. The owner's test (JOB.md, 2026-10-24: "For every name, point at the word in the line that says what it is ... If no word says it, the proposal hides meaning and fails") and the lead's N79 ("`owns` is the one word that ends what it stands before, so it is never an operand; the other words make a value and keep their meaning") both point the same way as the reader: an operator reads. Python's `a + b` leaves `a` usable, which is what a Python programmer expects of `full = first + " Lovelace"`. So the reader's direction is right and at the cause; it is narrowed because (1) the status is wrong as a description (the text decides it, inconsistently), (2) the edit leaves `a + b` with `b: String` unsaid (operands are not in TYP-5's list of coercion sites, so an implementer needs TXT-8's "A `String` place reads as a `StrView` wherever one is expected" named), and (3) the new allocation per `+` needs its place in the X.4 row that already covers "a `String` built" (COST-5: "A rule that introduces an implicit cost MUST add a row"). No verdict covers an operator's operand (JOB.md has no verdict on text `+`; the search found only `+=` in reductions).

- BETTER EDIT rule=TXT-11
  - old: `a + b` for `a: String, b:
  StrView` takes `a` over and returns it extended; `s += t` appends. `StrView + StrView` is `E2040`
  whose help is an f-string.
  - new: `a + b` for `a: String, b: StrView` reads both operands in place and gives a new `String`, `a`'s text then `b`'s, in one allocation (`String` implements `Add[StrView, Output = String]`, `[TYP-21]`; a `b: String` is read as a `StrView`, `[TXT-8]`), so `a` stays usable, as in Python; `s += t` appends to `s` in place (`AddAssign[StrView]`) and allocates only to grow, which is how text is built in a loop (`s = s + t` copies `s` each round). `StrView + StrView` is `E2040` whose help is an f-string.
  - why: Every operator then reads its operands, as GRM-47 and the E0119 row already claim; the hand-over with no word, which the owner's test refuses and N79 forecloses adding a word to, is gone; the right operand's type is said, since an operand is not one of TYP-5's coercion sites.

- BETTER EDIT rule=TYP-21
  - old: Operators desugar to
  these interfaces for non-scalar operands, but one: `a + b` calls `Add.add(a, b)` with both
  operands read in place, and `String`'s `+` is the language's own, taking `a` over (`[TXT-11]`), so
  `String` implements no operator interface and does not meet `T: needs Add`.
  - new: Operators desugar to these interfaces for non-scalar operands: `a + b` calls `Add.add(a, b)` with both operands read in place, for `String` too (`String` implements `Add[StrView, Output = String]` and `AddAssign[StrView]`, `[TXT-11]`, so it meets `T: needs Add[StrView, Output = T]`, and `a + b` on it makes a new `String`).
  - why: The exception goes with TXT-11's change; the reader's text, kept.

- BETTER EDIT rule=TYP-21
  - old: `bool`, `char` and `String` implement none of them.
  - new: `bool` and `char` implement none of them.
  - why: `String` now implements `Add` and `AddAssign`; the reader's text, kept.

- BETTER EDIT rule=§X.4 row "Allocation"
  - old: | Allocation | guaranteed present | a call that allocates (a collection growing, a `String` built), reported per site; `@noalloc` refuses it (`[EFF-1]`) |
  - new: | Allocation | guaranteed present | a call that allocates (a collection growing, a `String` built, `a + b` on a `String`, `[TXT-11]`: one allocation, where `s += t` grows in place), reported per site; `@noalloc` refuses it (`[EFF-1]`) |
  - why: COST-5 asks a row of every implicit cost; the allocation `+` now makes is named where allocations are priced, and `@noalloc`'s refusal of it (`E4001`) follows from the row.

## K28-untyped-list-of-text  reader=answered_differently  refuter=confirmed  should_be=answered_differently  safety=none

**answered_by**: TYP-15: "`names = ["ann", "bob"]` is an `Array[StrView]` of text written in the program, which may live as long as the program"; TYP-38: "A list literal `[a, b, c]` has the type its context expects: `Array[T]` (allocates), `[T; N]` (no allocation) or `Span[T]` (a view of a temporary fixed array). With no context it is `Array[T]`."; TYP-38: "With no context a text key, value or element is a `String`, since a map written with no type owns its text; it holds `StrView` only where the program says so (`labels: Map[StrView, int]`, `[STD-11]`)."; LEX-20: "A string literal has type `StrView` with the static region: a view of text written in the program. At a site expecting `String` it produces a `String` (`[TXT-9]`)."; TYP-23: "a later use that needs another type is `E2020`, whose help names the declaration and the annotation (`total: i32 = 0`)"

**program**: Decided as the checker's option B: `names = ["ann", "bob"]` is an `Array[StrView]` (TYP-15 says so in those words), `seen = {"ann", "bob"}` a `Set[String]` (TYP-38), and `names.push(owns newcomer)` with `newcomer: String` is `E2020`, whose help names `names: Array[String] = ["ann", "bob"]` (TYP-23).

- READER EDIT rule=TYP-38
  - old: With no context it is `Array[T]`.
  - new: With no context it is `Array[T]`, `T` the elements' own type: `["ann", "bob"]` is an `Array[StrView]` of text written in the program, allocating no `String` (`[TYP-15]`), unlike a map or set literal (below); a list that is to own its text says so (`names: Array[String] = ["ann", "bob"]`).
  - why: The list bullet never says what `T` is for text, and the next bullet's "a text key, value or element is a `String`" can be read as covering a list's elements; the one example that decides it (TYP-15) is in another Part. Stating it where literals are defined removes the reading that made the finding.

**remaining_doubt**: The asymmetry (a list keeps views, a map or set owns its text) is a design choice the text states on both sides and no verdict touches; it is safe either way and the current choice is the faster one (no allocation per element), in line with the owner's "smallest tool" rule, and the finder's Python-style program gets a message naming the one line to write. If the lead prefers the finder's A, the cost is one allocation per literal element and TYP-38's `@noalloc` sentence already covers it (`E4001`).

**REFUTER reason**: The text decides it as the reader says: TYP-15 "`names = [\"ann\", \"bob\"]` is an `Array[StrView]` of text written in the program"; LEX-20 "A string literal has type `StrView` with the static region ... At a site expecting `String` it produces a `String`"; TYP-38 "With no context it is `Array[T]`"; TYP-23 "An untyped literal is never left open: its context is the type expected at the literal itself, and an unannotated declaration supplies none, so `total = 0` declares an `int` whatever the later uses (ODR-022); a later use that needs another type is `E2020`, whose help names the declaration and the annotation" (so no later `push` changes the element type, and `names.push(owns newcomer)` is `E2020`). The only reading that made the finding is TYP-38's second bullet, "With no context a text key, value or element is a `String`, since a map written with no type owns its text", whose "element" sits in the map-and-set bullet but is not said to be a set's; the reader's edit says in the list bullet what TYP-15 says in another Part. The edit opens no hole, adds no cost (it states the allocation-free choice already made) and touches no verdict. Confirmed as written; the asymmetry (list of views, set and map of owned text) is a stated design choice, not a defect.

## K30-shared-owner-equality-and-identity  reader=not_answered  refuter=confirmed  should_be=not_answered  safety=none

**answered_by**: TYP-14: "A reference, and an `owns T`, is read through wherever a `T` is wanted — a field access, a method call, an operand, an argument to a plain parameter."; TYP-5 rule 7: "`ref T` from a place (a read-only view of it), and reading through a `ref` or an `owns T` where a `T` is expected (`[TYP-14]`);"; IV.11 step 1: "Strip views from `recv`'s type `R` (`ref`, `ref mut`, `sharedref mut`), and one level of `owns`, `shares`, `shares mut` or `thread_shared`."; TYP-36: "A read-only view `ref T` implements `Eq`, `Ord`, `Hash`, `Display` and `Debug` as `T` does (so `sorted(m.keys())` works). An `owns T` is `Clone` when `T` is ... Two `owns` values compare with `==` when what they hold does"; EXP-9: "`a is b` tests whether two class objects (or two views) are the same object, by comparing addresses. ... Any other operand of `is` is `E2150`, whose help names `==`."; registry E2150: "`is` on operands that are neither class objects, links, references nor `Option`s, and not a kind test (`x is T`)"; VI.3 table: "| `a is b`, `a is not b` | the same object | class objects and views (`[EXP-9]`) |"

**program**: fn total(xs: Span[int]) -> int:
    return sum(xs)

class Enemy:
    hp: int = 10

struct Hud:
    target: weakref Enemy = None

fn same_target(cam: Hud, hud: Hud) -> bool:
    return cam.target is hud.target      # EXP-9: E2150 ("Any other operand"); the E2150 registry row admits links: undecidable

fn main():
    items = [1, 2, 3]
    other = shares mut items
    other.push(owns 4)                   # IV.11 strips one level: fine
    println(total(other))                # TYP-14/TYP-5 rule 7 name only `ref` and `owns T`: is a `shares mut Array[int]` read through to a `Span[int]`? unsaid
    println(other)                       # TYP-36 gives `ref T` and `owns T` the interfaces of T; nothing gives them to an owner: Printable? unsaid
    println(other == [1, 2, 3, 4])       # unsaid: contents (Rust's `Rc`) or identity (C++'s `shared_ptr`)
    a = shares items
    println(a is other)                  # EXP-9: E2150; the registry row: accepted? undecidable

- READER EDIT rule=TYP-14
  - old: A reference, and an
  `owns T`, is read through wherever a `T` is wanted — a field access, a method call, an operand, an
  argument to a plain parameter.
  - new: A reference, an `owns T`, and a `shares T`, `shares mut T` or `thread_shared T` owner, is read through wherever a `T` is wanted — a field access, a method call, an operand, an argument to a plain parameter (a `thread_shared T` as a read, which `[THR-1]` allows every owner; the owner word says who owns the value, not what it is).
  - why: IV.11, RC-2a and GRM-44 (`weakref thread_shared T` "reads the value as a parameter `p: thread_shared T` does") already assume that an owner reads as its value; only TYP-14 and TYP-5 rule 7 leave the shared owners out, so an argument or an operand through one is undecided.

- READER EDIT rule=TYP-5 rule 7
  - old: 7. `ref T` from a place (a read-only view of it), and reading through a `ref` or an `owns T` where
     a `T` is expected (`[TYP-14]`);
  - new: 7. `ref T` from a place (a read-only view of it), and reading through a `ref`, an `owns T` or a `shares`, `shares mut` or `thread_shared` owner where a `T` is expected (`[TYP-14]`);
  - why: The coercion list is "the complete list"; it must name what TYP-14 reads through.

- READER EDIT rule=TYP-36
  - old: A read-only view `ref T` implements `Eq`,
  `Ord`, `Hash`, `Display` and `Debug` as `T` does (so `sorted(m.keys())` works).
  - new: A read-only view `ref T`, and a `shares T`, `shares mut T` or `thread_shared T` owner, implements `Eq`, `Ord`, `Hash`, `Display` and `Debug` as `T` does, on the value it views or owns (so `sorted(m.keys())` works, `a == b` on two owners compares the two values, and `println(scores)` prints the list as `[10, 20, 30]`); whether two owners own one value is `a is b` (`[EXP-9]`), and a class object's `==` stays its identity.
  - why: Nothing gave an owner the interfaces of its value, so `==`, a map key and printing through an owner were undecided; the value's meaning under every owner word is the nearest rule (the one `owns T` has) and keeps one meaning of `==`, with `is` already spelling identity.

- READER EDIT rule=EXP-9
  - old: `a is b` tests whether two class
  objects (or two views) are the same object, by comparing addresses.
  - new: `a is b` tests whether two class objects, two views, two owners (`shares`, `shares mut`, `thread_shared`) or two links (`weakref`, `weakref thread_shared`, `sharedref mut`) name the same object or value, by comparing addresses; two links that hold nothing are `is` each other, and a link whose target has ended still names its block, which stays while a link points at it (`[WK-11]`), so the compare is exact and reads nothing of the target. A class object's `==` is this test (`[TYP-36]`).
  - why: The rule and the E2150 registry row decide `cam.target is hud.target` in opposite ways (the rule refuses, the row admits links); the row's answer is the general one and each case is one address compare that needs no opening, which the opened form (`exists as` on both) cannot give once the target has ended.

- READER EDIT rule=VI.3 table row `a is b`
  - old: | `a is b`, `a is not b` | the same object | class objects and views (`[EXP-9]`) |
  - new: | `a is b`, `a is not b` | the same object | class objects, views, owners and links (`[EXP-9]`) |
  - why: The table must claim what the rule says.

- READER EDIT rule=registry row E2150
  - old: `is` on operands that are neither class objects, links, references nor `Option`s, and not a kind test (`x is T`)
  - new: `is` on operands that are neither class objects, owners (`shares`, `shares mut`, `thread_shared`), links, references nor `Option`s, and not a kind test (`x is T`)
  - why: One code, one defining rule: the row then matches EXP-9 exactly.

**remaining_doubt**: Safety is not at stake in either direction (`is` is an address compare on memory a link keeps, WK-11; reading through an owner is what IV.11 already does for a method call). The choice of `==` on owners comparing values rather than identity is the finder's A; the lead may prefer to put it to the owner, but the text's own `owns T` rule ("Two `owns` values compare with `==` when what they hold does") points at A.

**REFUTER reason**: Confirmed on both halves. (1) Links under `is`: EXP-9 "`a is b` tests whether two class objects (or two views) are the same object, by comparing addresses ... Any other operand of `is` is `E2150`, whose help names `==`" refuses `cam.target is hud.target`, while the E2150 row "`is` on operands that are neither class objects, links, references nor `Option`s" admits links: one code, two answers, so an implementer cannot decide the line. (2) Owners: TYP-14 "A reference, and an `owns T`, is read through wherever a `T` is wanted" and TYP-5 rule 7 "reading through a `ref` or an `owns T` where a `T` is expected" name no `shares`, `shares mut` or `thread_shared` owner, and TYP-5 is "the complete list", so `total(other)` for `other: shares mut Array[int]` and `fn total(xs: Span[int])` is accepted by nothing; IV.11 step 1 strips "one level of `owns`, `shares`, `shares mut` or `thread_shared`" for a method call only; TYP-36 gives `ref T` and `owns T` the value's interfaces ("A read-only view `ref T` implements `Eq`, `Ord`, `Hash`, `Display` and `Debug` as `T` does ... Two `owns` values compare with `==` when what they hold does") and TYP-39 says "An `owns T` and a `Cell[T]` show as the `T` they hold", so by their silence a `shares` owner has no `Eq`, `Hash` or `Display` and is in no table row: `println(other)` and `other == [1, 2, 3, 4]` are undecided (`E2040` by the table's "—" would need a row). The reader's edits follow the nearest rules (`owns T`'s, and THR-1's "`log: thread_shared Log` reads the caller's value with no count"), keep one meaning of `==` (values; a class object's stays identity, as the TYP-36 table says), and spell identity with `is`, which WK-11 makes exact ("Every `weakref` link holds one weak count on its target", so the block stays). A `shares mut` key changed through another owner is HASH-3's case ("wrong answers ... but never undefined behaviour"). No verdict touches `==` or `is` on owners (JOB.md's `is` verdicts concern the kind test). No cost is added: a read-through is how the value is reached, and `is` is one compare.

## K41-some-after-conversions  reader=answered_differently  refuter=narrowed  should_be=answered_differently  safety=none

**answered_by**: TYP-5: "Each rule composes with the others only where stated (widening after range erasure, and rule 11 after rules 1–4). Nothing converts implicitly to or from `bool` or `char`."; TYP-5 rule 11: "a value of type `T` to `Option[T]`, as `Some(value)`, where `T` is not itself an `Option`: one level only, so nothing becomes `Option[Option[T]]` (`return v` in a function returning `Option[T]`, `f(5)` for a parameter `x: Option[int] = None`);"; TYP-5 rule 12: "After `n = 5`, `y: u64 = n` is accepted; `y: u64 = x` for a parameter `x: int` is `E2020`."; GRM-13: "a number, a `bool` or a variant with no payload is copied, as Python binds a number"

**program**: Decided as the checker's option B: `cap: Option[u64] = n` after `n = 5` (rule 12 then 11), `slot: Option[owns dyn Weapon] = owns Sword(edge=4)` (rule 8 then 11) and `on_hit: Option[Handler] = log_hit` (rule 10 then 11) are `E2020`, and the lines compile with `Some(...)` written, as verdict 85's own example does (`left=Some(owns Tree(value=2))`). The finder's `return w` for a loop name over `Span[int]` is rule 11 alone, since GRM-13/CTL-1 copy a number into a loop name.

- READER EDIT rule=TYP-5
  - old: Each rule composes with the others only where stated
      (widening after range erasure, and rule 11 after rules 1–4).
  - new: Each rule composes with the others only where stated: widening after range erasure; rule 7, a read through a view or an owner, before any rule, since it converts nothing (`n: ref String` given to a `StrView` parameter, an `owns Array[int]` to a `Span[int]`); and rule 11 after any one of rules 1 to 10 and 12, so an `Option[T]` place accepts as `Some` whatever a `T` place accepts (`cap: Option[u64] = n` after `n = 5`; `slot: Option[owns dyn Weapon] = owns Sword(edge=4)`), still one level only.
  - why: Read literally, the sentence also refuses rule 7 followed by rule 4 or 6 (`n = p.name` then `greet(n)` for `fn greet(s: StrView)`; `f(b)` for `b: owns Array[int]` and `fn f(xs: Span[int])`), which TYP-14 ("read through wherever a `T` is wanted") and TXT ("A `String` place reads as a `StrView` wherever one is expected") appear to accept, so an implementer has two readings of the commonest lines; stating that a read-through is not a conversion settles them, and letting `Some` follow any one rule gives `x: T = v` and `x: Option[T] = v` the same accepted values with no list to maintain.

**remaining_doubt**: The `Option` half is safe as it stands (a refusal with `Some(...)` as the fix) and is the lead's to keep or change; the edit is offered because the same sentence leaves the read-through compositions (rule 7 then 4 or 6) in doubt, which is a real implementer question outside the cluster's own program and the reason to touch the sentence at all.

**REFUTER reason**: The Option half is decided as the reader says: TYP-5 "Each rule composes with the others only where stated (widening after range erasure, and rule 11 after rules 1–4)", so `cap: Option[u64] = n` (rule 12 then 11), `slot: Option[owns dyn Weapon] = owns Sword(edge=4)` (rule 8 then 11) and `on_hit: Option[Handler] = log_hit` (rule 10 then 11) are `E2020`, and the lines compile with `Some(...)`; verdict 85's `left=Some(owns Tree(value=2))` writes `Some` but rules nothing on it. The reader's edit then reverses what it calls decided, which is why this is narrowed, not confirmed; but the reversal is defensible at the cause: the list "rules 1–4" predates rule 12 (added with the owner's ruling of 2026-10-06) and rule 8's words, so `cap: Option[u64] = 5` is accepted (the literal takes `u64`, then rule 11) while `cap: Option[u64] = n` after `n = 5` is refused, a difference no rule intends; "an `Option[T]` place accepts as `Some` whatever a `T` place accepts" is the general statement, safe (each composed rule is itself safe and one level of `Some` is kept) and costless. The read-through doubt is real: rule 7 is in the list of coercions, so "only where stated" literally refuses rule 7 then rule 4 (`greet(n)` for `n: ref String`, `fn greet(s: StrView)`) and rule 7 then rule 6 (`f(b)` for `b: owns Array[int]`, `fn f(xs: Span[int])`), while TYP-14 ("read through wherever a `T` is wanted — ... an argument to a plain parameter") and TXT-8 ("A `String` place reads as a `StrView` wherever one is expected") accept them; saying that a read-through converts nothing and precedes any rule settles it. The reader's edit is kept; status is answered_differently because the current text decides the programs, and the edit is a lead's design choice, not a hole's closing.

- BETTER EDIT rule=TYP-5
  - old: Each rule composes with the others only where stated
      (widening after range erasure, and rule 11 after rules 1–4).
  - new: Each rule composes with the others only where stated: widening after range erasure; rule 7 before any rule, since a read through a view or an `owns T` converts nothing but reaches the value (`greet(n)` for `n: ref String` and `fn greet(s: StrView)`, rule 7 then 4; `f(b)` for `b: owns Array[int]` and `fn f(xs: Span[int])`, rule 7 then 6); and rule 11 after any one of the other rules, so an `Option[T]` place accepts as `Some` whatever a `T` place accepts (`cap: Option[u64] = n` after `n = 5`; `slot: Option[owns dyn Weapon] = owns Sword(edge=4)`; `on_hit: Option[Handler] = log_hit`), still one level only.
  - why: The read-through sentence removes the two readings of the commonest lines; the rule-11 sentence replaces a list that was not kept up to date when rules 8 to 12 were added, with the property instead of the list, which is what rule 7 of the brief asks.

## K45-dyn-with-self-parameter  reader=answered  refuter=confirmed  should_be=answered  safety=none

**answered_by**: TYP-22: "An interface is usable as `dyn` only if every method has an object (none is a `static fn`), names `Self` nowhere but as `this`: not as a result by value, and not as a parameter's type or inside one (`fn eq(other: Self)`, `fn add(rhs: Self)`, `fn merge(owns other: Self)`), since through a `dyn I` value the kind of any other value is unknown (methods marked `where Self: needs Sized` excepted, which a `dyn` value does not have)"; TYP-22: ""`Same` cannot be used as `dyn Same`: `same` takes `other: Self`, and through a `dyn Same` the kind of `other` is unknown; take `other: dyn Same` and test its kind in the body, `if other is Tag as o:`, or mark `same` with `where Self: needs Sized` and call it on the concrete types""; TYP-22: "A method of an interface that `I` requires (`interface Shape: needs (Eq, Ord):`) is reached through `dyn I` only where it names `Self` nowhere but as `this`, and through the two entries this document gives: `clone` (§IX.1) and `eq` (`[TYP-36]`, the kinds compared first); any other such method called through a `dyn I` value is `E2050` at the call ("`cmp` takes `other: Self`, and through a `dyn Shape` the kind of `other` is unknown; test the kinds first ...")"; TYP-36: "The table is normative; a `—` means the implementation does not exist and a bound requiring it is `E2040`. ... An `owns T` is `Clone` when `T` is, and `owns dyn I` when `I` requires `Clone` ... `owns dyn I` compares when `I` requires `Eq` ... that entry is the one way `Eq` is reached through `dyn`"

**program**: 

**remaining_doubt**: The finder's `check(a: dyn Collider, b: dyn Collider)` is `E2050` at `dyn Collider` (the finder's A, decided when compiling). Sorting an `Array[owns dyn Shape]` with `interface Shape: needs Ord:` is `E2040`: TYP-36 grants `owns dyn I` only `Clone` and `Eq`, and TYP-22 makes `cmp` through `dyn` `E2050` at the call; the interface itself stays usable as `dyn`, a slightly different cut from the finder's A ("not usable as `dyn`") but safe and decided. One clause in TYP-36 ("`owns dyn I` has no `Ord` or `Hash`") would spare an implementer the inference from a missing row.

**REFUTER reason**: TYP-22 decides `check(a: dyn Collider, b: dyn Collider)`: "An interface is usable as `dyn` only if every method ... names `Self` nowhere but as `this`: not as a result by value, and not as a parameter's type or inside one (`fn eq(other: Self)` ...) ... Violations are `E2050`", with the message to write (`other: dyn Same` and a kind test, or `where Self: needs Sized`). Sorting an `Array[owns dyn Shape]` with `interface Shape: needs Ord:` is decided by TYP-22's "A method of an interface that `I` requires ... is reached through `dyn I` only where it names `Self` nowhere but as `this`, and through the two entries this document gives: `clone` (§IX.1) and `eq` (`[TYP-36]`) ... any other such method called through a `dyn I` value is `E2050` at the call ('`cmp` takes `other: Self` ...')" and TYP-36's "The table is normative; a `—` means the implementation does not exist and a bound requiring it is `E2040`" with its prose granting `owns dyn I` only `Clone` and `Eq`. Both refusals are safe. The reader's remaining doubt is right that `owns dyn I` has no table row, so the `E2040` is read from an omission; the clause goes under `missed`.

## K59-runtime-placement-beyond-verdicts  reader=answered_differently  refuter=confirmed  should_be=answered_differently  safety=none

**answered_by**: LT-48: "`runtime` is written on a parameter or on a local declared with its type, after the `:` and before `dyn` (under any holding word: `dyn I`, `ref dyn I`, `owns dyn I`, `shares dyn I`, `shares mut dyn I`) or before the name of an `open` or `abstract` class (under any holding word likewise): `fn show(p: runtime dyn Picker):`; `fn show(a: runtime A):` for `open class A`; `q: runtime owns dyn Picker = owns Mine(items=[9]) if input() == "m" else owns Yours(items=[9])`; `me: runtime ref A = ref this`. It has one meaning wherever it stands."; LT-48: "for a value with no name of its own (`ps[0].pick(xs)`, `this.picker.pick(xs)`, `make().pick(xs)`), "to check it while running, name the value with `runtime` and call through the name: `p: runtime ref dyn Picker = ref this.picker`, then `r = p.pick(xs)`""; LT-48: "`runtime` is written nowhere else. ... on a field, a result type, a type argument, a capture or a local declared without its type it is `E0100` ("`runtime` goes on the parameter or the local a call goes through, so that the function whose line pays says so, not on a field; in the method that calls through `picker`, name its value first: `p: runtime ref dyn Picker = ref this.picker`" ...)"; grammar param / var_decl: "param         := [param_word] IDENT ":" ["runtime"] type ["..."] ["=" expression] ... var_decl      := IDENT ":" ["runtime"] type ["=" expr_list]        -- `runtime` before `dyn` or an open class ([LT-48])"; registry E3021: "a view a `dyn` call may give included where the kind is known only while running (help: `runtime` on the parameter or the typed local the call goes through, or a `runtime` name for the value)"; decisions.md N78: "A `runtime` type or field was refused: the function whose line pays says so."

**program**: Decided as the checker's option (b), by the lead's ruling N78: the finder's `renderer: runtime owns dyn Renderer` field is `E0100` with the local line as its help, and `render` writes `r: runtime ref dyn Renderer = ref this.renderer` then `frame = r.draw(this.scratch)`; the word's position before a holding word is fixed by the grammar and the examples (`p: runtime owns dyn Picker`, not `owns runtime dyn Picker`); `runtime` before `shares dyn I` and `shares mut dyn I` is named. Verdicts 162 and 163 are met (a class type; a variable "right after the colon").

- READER EDIT rule=grammar lambda_param
  - old: lambda_param  := [param_word] IDENT [":" type]
  - new: lambda_param  := [param_word] IDENT [":" ["runtime"] type]      -- `runtime` as on a `param` ([LT-48])
  - why: LT-48 and LEX-15 give the word to "a parameter", and a lambda's typed parameter is one whose body calls through it; the grammar alone leaves `fn(p: runtime dyn Picker) => ...` an unlisted `E0100`, and the E0100 list of refused places does not name it, so an implementer cannot tell whether the omission is meant.

**remaining_doubt**: The finder's recommendation (a), the word on the field, is refused by N78 with a stated reason (the function whose line pays says so); it is a lead ruling, not an owner verdict, and safety does not decide it (both shapes make the same check). Nothing else of the cluster is open.

**REFUTER reason**: The placement is decided by LT-48 ("`runtime` is written on a parameter or on a local declared with its type, after the `:` and before `dyn` (under any holding word ...) or before the name of an `open` or `abstract` class ... `runtime` is written nowhere else ... on a field, a result type, a type argument, a capture or a local declared without its type it is `E0100`"), the grammar (`param := [param_word] IDENT ":" ["runtime"] type`, `var_decl := IDENT ":" ["runtime"] type`) and the lead's N78 ("A `runtime` type or field was refused: the function whose line pays says so"); verdicts 162 ("`runtime` goes before a class type too") and 163 ("`runtime` goes on a variable too, right after the colon") are met. The lambda edit is a real inconsistency, correctly placed: LT-48 and LEX-15 ("`runtime` after the `:` of a parameter or of a local declared with its type") give the word to "a parameter" and the E0100 list does not name a lambda's parameter, while `lambda_param := [param_word] IDENT [":" type]` cannot produce it; the grammar refuses today, so the line is decided but the prose promises otherwise. Allowing it keeps the feature (the owner's condition) and uses LT-48's existing mechanism for a parameter whose callers are not all known ("every kind that implements the interface or descends from the class is taken to reach the name"), at LT-48's priced cost ("one load at the call and one bit test at each line it guards", already a §X.4 row).

## K62-copy-of-on-loop-names  reader=answered_differently  refuter=confirmed  should_be=answered_differently  safety=none

**answered_by**: grammar for_stmt / pattern: "for_target    := pattern {"," pattern} ... primary_pattern := "_" | literal_pattern | range_pattern | IDENT | path "(" ... | "[" [pattern {"," pattern}] ["," ".." [IDENT]] "]" | "ref" ["mut"] IDENT"; grammar opened_name: "opened_name   := [name_word] IDENT | "(" opened_name {"," opened_name} [","] ")" ... name_word     := "ref" ["mut"] | "copy" "of""; GRM-13: "A name a pattern binds in a place it does not take over is a read-only view of the part it names, as a name for a part is (Part VII): a number, a `bool` or a variant with no payload is copied, as Python binds a number, and nothing is changed through the name (`E3023`). `ref mut` before it makes a name for that part that may change it"; CTL-1: "each loop name is a read-only view of the element it is on, and what it reads does not change while it is used (`[BRW-1]`); `for ref mut x in e:` names the element so that it may change it"; GRM-44: "`if scores.get(name) exists as copy of v:` is a copy of the program's own, which may be changed (`v += 1`; the map is untouched), the same as `if scores.get(name) exists as v:` then `mine = copy of v`"

**program**: Decided as the checker's option (b): `for copy of e in enemies:` and the arm `Some(copy of v):` are not produced by the grammar (a pattern's words are `ref` and `ref mut` only), so each is `E0100`, and the two-line form (`for e in enemies:` then `mine = copy of e`) is the way, as verdict 66 names it for `exists as`.

**remaining_doubt**: Safety does not decide it (a `copy of` name is a copy either way) and verdicts 66 and 67 speak of `exists as` only; adding `copy of` to loop and pattern names (the finder's (a)) is a design choice for the lead or the owner. If taken, the edit is one grammar line (`primary_pattern` gains `"copy" "of" IDENT`) and one clause in GRM-13 giving it GRM-44's meaning (a copy by the type's `Clone`, changeable, `E2116` where `copy of` is), with no run-time cost beyond the copy the line shows.

**REFUTER reason**: The grammar decides it: `for_target := pattern {"," pattern}`, `match_arm := pattern ["if" expression] ":" block`, and `primary_pattern` ends with `"ref" ["mut"] IDENT` and has no `copy of`; `copy of` is a `name_word` only for `opened_name` ("`name_word := "ref" ["mut"] | "copy" "of"`", used after `exists as` and `is T as`). So `for copy of e in enemies:` and `Some(copy of v):` are not produced and are `E0100`. GRM-13 ("A name a pattern binds in a place it does not take over is a read-only view ... a number, a `bool` or a variant with no payload is copied") and CTL-1 ("each loop name is a read-only view of the element it is on") give the names their meaning, and GRM-44 gives the two-line form (`exists as v` then `mine = copy of v`) that verdict 66 made valid; GRM-27 shows the same for a comprehension (`[copy of s for s in shapes ...]`). Verdicts 66 and 67 speak of `exists as` only. Safe either way; nothing to add.

## K64-list-pattern-arms  reader=answered_differently  refuter=narrowed  should_be=answered_differently  safety=none

**answered_by**: GRM-40: "a bracket line not directly above one of them is `E0100`: a blank line or a comment line between `[T]` and its `fn` is `E0100` too, and so is every line that begins with `[` anywhere else, one that begins with a list included (`[10, 20].len()` on a line of its own; `n = [10, 20].len()` is the way)"; GRM-40: "A line that begins with `[` is a type-parameter line when the line after it begins a function, struct or class declaration (with `pub`, `open`, `abstract`, `static`, `virtual`, `override`, `gen`, `unsafe` or `extern` before it, or none); this is the one place where the parser looks past a line's end, at the next line's first tokens."; grammar III.6: "primary_pattern := ... | "[" [pattern {"," pattern}] ["," ".." [IDENT]] "]" | "ref" ["mut"] IDENT"; grammar match_arm: "match_arm     := pattern ["if" expression] ":" block"; JOB.md verdict 155: "Every line that starts with `[` and is not directly above a function, struct or class is `E0100`"

**program**: The headline question is decided against the finder: an arm `["quit"]:` begins a line with `[` and is `E0100` under GRM-40 and verdict 155 (a list pattern stands only nested, `Some(["quit"]):`). What an implementer still cannot answer is the nested pattern's meaning, which no rule gives:

fn head(words: Option[Span[StrView]]) -> StrView:
    match words:
        Some([first, ..rest]):      # grammatical; what does it match (a Span? an Array? a String?), and what are `first` and `rest`?
            return first
        _:
            return ""

`[..rest]` with nothing before the `..` is not even producible (the grammar requires a comma before `..`).

- READER EDIT rule=GRM-13
  - old: 
  - new: A list pattern `[p, q]` matches a `Span[T]`, `MutSpan[T]`, `Array[T]` or `[T; N]` of exactly that many elements and binds each name to its element as above; `[p, q, ..]` matches one with at least that many, and `[p, ..rest]` names the remaining elements `rest`, a `Span[T]` of them (a `MutSpan[T]` where the scrutinee is a `ref mut` name of the list), a view like every pattern name, which the list's rules govern (`[BRW-1]`); `[..rest]` alone names the whole. A list pattern against a `String` or `StrView` is `E2020`, whose help gives `split`.
  - why: The grammar produces list patterns and nothing says what they match or what `..rest` binds; the meaning follows the pattern default (names are views) and the one view type Ember has for a run of elements.

- READER EDIT rule=grammar primary_pattern
  - old:                | "[" [pattern {"," pattern}] ["," ".." [IDENT]] "]"
  - new:                | "[" [pattern {"," pattern}] ["," ".." [IDENT]] "]" | "[" ".." [IDENT] "]"   -- a list pattern ([GRM-13])
  - why: `[..rest]` and `[..]` are the forms a non-empty-or-empty test is written with and the grammar cannot produce them.

**remaining_doubt**: The current text meets verdict 155 to the letter, and refusing is safe. The finder's (a), a list pattern as an arm at the top level, needs the owner, since it narrows a verdict: inside a match's arm list no type-parameter line and no statement can stand, so the refusal there is a side effect of a verdict put to the owner with a stray expression and a bracket above an enum; the recommended owner question is whether the line after the `[` line being an indented block (an arm's body) rather than a declaration makes it an arm. Until he rules, option (b) stands and the lead should at least give the nested form its meaning (the edits above), or remove the list pattern from the grammar (option (c)), which drops a feature and so is the worse shape under the delegation's condition.

**REFUTER reason**: The headline is decided as the reader says: GRM-40 "so is every line that begins with `[` anywhere else, one that begins with a list included" and verdict 155 "Every line that starts with `[` and is not directly above a function, struct or class is `E0100`" refuse an arm `["quit"]:`; a list pattern stands nested (`Some(["quit"]):`) or on a `for` line. The nested hole is real: the grammar produces `"[" [pattern {"," pattern}] ["," ".." [IDENT]] "]"`, and no prose in the document names a list pattern, what it matches or what `..rest` binds (a search for list patterns, `..rest`, `[..]` and `[_, ` finds nothing but the grammar line); `[..rest]` is not producible, since the comma before `..` is required. Narrowed only in the fix: the reader's GRM-13 text makes `rest` a `Span[T]` in every case, but under `match owns xs:` (GRM-13: "`match owns e:` hands `e` over, and each name of the branch that runs owns its part") a view of a list that ends with the match is a dangling view the rule would then have to catch with `E3063`; it is cleaner to say that `..rest` reads its list in place. The reader's text also leaves a pattern of `n` names with no `..` against a `[T; N]` with `N != n` undecided (a test that can never pass); `E2020` names both types. The grammar edit is kept.

- BETTER EDIT rule=GRM-13
  - old: 
  - new: A list pattern `[p, q]` matches a `Span[T]`, `MutSpan[T]`, `Array[T]` or `[T; N]` of exactly that many elements and binds each name to its element as above (`N` that can never equal the count is `E2020`, naming both); `[p, q, ..]` matches one with at least that many; `[p, ..rest]` names the remaining elements `rest`, a `Span[T]` of them (a `MutSpan[T]` where the scrutinee is a `ref mut` name of the list), a view like every pattern name, which the list's rules govern (`[BRW-1]`), and `[..rest]` alone names the whole; a `..rest` name reads its list in place, so under `match owns xs:` it is `E2020`, whose help is to match `xs` in place and take what is needed with `copy of rest` or `xs.drain(1..)`. A list pattern against a `String` or `StrView` is `E2020`, whose help gives `split`.
  - why: The grammar produces list patterns and nothing says what they match or binds; the meaning follows the pattern default (names are views) and the one view type Ember has for a run of elements, and the two cases the reader's text left open (a fixed array of the wrong size, a `..rest` view of a list the match ends) are closed safely.

- BETTER EDIT rule=grammar primary_pattern
  - old:                | "[" [pattern {"," pattern}] ["," ".." [IDENT]] "]"
  - new:                | "[" [pattern {"," pattern}] ["," ".." [IDENT]] "]" | "[" ".." [IDENT] "]"   -- a list pattern ([GRM-13])
  - why: `[..rest]` and `[..]` are the forms an empty-or-not test is written with, and the grammar cannot produce them; the reader's edit, kept.

## K65-several-interfaces-after-dyn  reader=not_answered  refuter=confirmed  should_be=not_answered  safety=none

**answered_by**: grammar III.2/III.3: "needs         := type | "(" type {"," type} [","] ")"     -- one bare, several in round brackets ([TYP-17])
bound_list    := type {"+" type}                        -- after `dyn` and `some` only ... | "dyn" bound_list | "some" bound_list"; TYP-17: "A bound without `needs`, or needs joined with `+`, is `E0100`, whose message gives the line with `needs` (`[T: Add + Copy]` gives `[T: needs (Add, Copy)]`)."; IV.9: "`dyn I` is unsized. ... It is a data pointer and a table pointer (a class object reaches its table through the object header, `[OBJ-2]`)."; DSP-3: "A call through a `dyn I` value of a class object finds the interface's table through the type index and caches the lookup for repeated calls on one value."; JOB.md verdict 109: "The same spelling goes everywhere a list of needs is written: verdicts 110 to 112."

**program**: The grammar accepts `+` after `dyn` and `some` (the only place `+` survives) and no rule says what a value of two interfaces is:

interface Shape:
    fn area() -> float

shapes: Array[owns dyn Shape + Debug] = [owns Square(side=2.0)]

fn show(s: dyn Shape + Debug):
    println(s.area(), f"{s!r}")        # which table does the call go through? `dyn I` has one table pointer; a value of two interfaces has no layout in the text, so a package cannot lay it out or write the call

fn visible(xs: Span[int]) -> some Iterator[Item = int] + Clone:
    return xs.iter().copy_each().filter(owns fn(x) => x > 0)

An implementer cannot answer: the layout of `dyn A + B` (one table pointer, two, or a combined table), nor whether `+` here is meant when TYP-17 refuses it everywhere else.

- READER EDIT rule=grammar bound_list
  - old: bound_list    := type {"+" type}                        -- after `dyn` and `some` only
  - new: bound_list    := type | "(" type {"," type} [","] ")"     -- after `dyn` and `some`: one bare, several in round brackets, as `needs` ([TYP-17], [TYP-22])
  - why: A list of interfaces a value must have has one spelling everywhere else (verdicts 109 to 112, "The same spelling goes everywhere a list of needs is written"); `+` here is the second spelling that consistency removes, and nothing in the text defines what `+` after `dyn` means.

- READER EDIT rule=TYP-17
  - old: A bound without `needs`, or needs joined with `+`, is `E0100`, whose
  message gives the line with `needs` (`[T: Add + Copy]` gives `[T: needs (Add, Copy)]`).
  - new: A bound without `needs`, or needs joined with `+`, is `E0100`, whose message gives the line with `needs` (`[T: Add + Copy]` gives `[T: needs (Add, Copy)]`); interfaces joined with `+` after `dyn` or `some` are `E0100` too, whose message gives the brackets (`owns dyn Shape + Debug` gives `owns dyn (Shape, Debug)`; after `dyn` and `some` the brackets stand without `needs`, the word before them saying what the value is).
  - why: The refusal and its line to write go where the rule for the spelling already is.

- READER EDIT rule=TYP-22
  - old: 
  - new: `dyn (A, B)` is a value of one type implementing every listed interface, each of which is itself usable as `dyn`, held with one data pointer and one table pointer as `dyn I` is: the table of the set, whose entries are `A`'s methods then `B`'s, each interface's in its declaration order, under one type-information symbol per set and type (`[MNG-4]`, `[TYP-44a]`), so every package lays it out and reads it alike (`[BLD-16]`), and a call through it is `[DSP-3]`'s; a kind test on it is one compare as on `dyn I`. `some (A, B)` is one concrete type meeting both, with no table (`[TYP-32]`). One interface takes no brackets (`owns dyn Shape`).
  - why: The text never says what a value of several interfaces is, so a package could not lay it out; the set's table keeps `dyn`'s two-pointer shape and the existing call path, with no new cost.

**remaining_doubt**: The spelling after `dyn`/`some` was never put to the owner; the edit rests on verdict 109's general sentence, and the lead may prefer to ask him with `[T: needs (Add, Copy)]` beside `owns dyn (Shape, Debug)`. The layout of a multi-interface value is an implementation fact the lead rules; the one given keeps `dyn` at two words and needs no run-time lookup beyond DSP-3's.

**REFUTER reason**: Confirmed: the grammar produces `dyn Shape + Debug` and `some Iterator[Item = int] + Clone` ("`bound_list := type {"+" type} -- after `dyn` and `some` only`"), and no rule in the document says what such a value is: §IV.9 defines `dyn I` as "a data pointer and a table pointer" for one interface, DSP-3 "finds the interface's table through the type index" for one interface, TYP-32 defines `some I` for one, and no prose line writes `dyn A + B` anywhere (a search finds none). TYP-17 refuses `+` for bounds ("needs joined with `+`, is `E0100`") and says nothing of `dyn`, so an implementer cannot tell whether the grammar's `+` is meant or a leftover. The reader's edits are sound: the spelling follows verdict 109's sentence "The same spelling goes everywhere a list of needs is written", though that verdict's own scope is `needs` lists (109 to 112), so extending it to `dyn`/`some` is the lead's ruling under the delegation (AUTOPILOT §3: "Record an owner's answer in exactly the owner's scope, never wider"), to be marked as such or put to the owner beside `[T: needs (Add, Copy)]`; `dyn (A, B)` has the same shape as `needs (A, B)` against a tuple type, which `needs` already resolves by meaning. The layout keeps `dyn` at two words and DSP-3's call, so no §X.4 row changes. Two things the reader's TYP-22 text leaves for an implementer to guess are added below: which package writes the set's table (BLD-16: "No fact of the whole program decides a package's code or layout"; the package that converts at TYP-5 rule 8's site knows both the type and the set, and MNG-4's naming lets the linker keep one), and whether a `dyn (A, B)` value goes to a `dyn A` place (TYP-5 rule 8 reads "a value to an interface its type implements", which through `dyn` is unknown, so without a word it is `E2020`).

- BETTER EDIT rule=grammar bound_list
  - old: bound_list    := type {"+" type}                        -- after `dyn` and `some` only
  - new: bound_list    := type | "(" type {"," type} [","] ")"     -- after `dyn` and `some`: one bare, several in round brackets, as `needs` ([TYP-17], [TYP-22])
  - why: The reader's edit, kept: one spelling for a list of interfaces, resolved against a tuple type by meaning as `needs` is.

- BETTER EDIT rule=TYP-17
  - old: A bound without `needs`, or needs joined with `+`, is `E0100`, whose
  message gives the line with `needs` (`[T: Add + Copy]` gives `[T: needs (Add, Copy)]`).
  - new: A bound without `needs`, or needs joined with `+`, is `E0100`, whose message gives the line with `needs` (`[T: Add + Copy]` gives `[T: needs (Add, Copy)]`); interfaces joined with `+` after `dyn` or `some` are `E0100` too, whose message gives the brackets (`owns dyn Shape + Debug` gives `owns dyn (Shape, Debug)`; after `dyn` and `some` the brackets stand without `needs`, the word before them saying what the value is).
  - why: The reader's edit, kept: the refusal and its line to write go where the rule for the spelling is.

- BETTER EDIT rule=TYP-22
  - old: 
  - new: `dyn (A, B)` is a value of one type implementing every listed interface, each of which is itself usable as `dyn`, held with one data pointer and one table pointer as `dyn I` is. The table of the set for a type is written by the package that converts a value of that type to `dyn (A, B)` (`[TYP-5]` rule 8), which sees both, under one symbol per set and type (`[MNG-4]`), so the linker keeps one copy (`[MONO-1]`) and every package lays it out and reads it alike (`[BLD-16]`): first the type's index (`[TYP-44a]`), then a pointer to each listed interface's own table for the type, then `A`'s entries and `B`'s, each interface's in its declaration order. A call through it is `[DSP-3]`'s, one indirect call; a kind test is one compare, as on `dyn I`; a `dyn (A, B)` value given to a `dyn A` place converts by `[TYP-5]` rule 8 with one load of the member's table pointer, and to a `dyn C` of an interface not listed it is `E2020`. `some (A, B)` is one concrete type meeting both, with no table (`[TYP-32]`). One interface takes no brackets (`owns dyn Shape`).
  - why: The reader's layout, with the two facts an implementer still had to guess: who writes the table (so that `[BLD-16]` holds) and what a value of several interfaces converts to; the member-table pointers make the conversion one load at the cost of a few words per table, nothing per call.

## K68-pack-tuple-sizes  reader=not_answered  refuter=confirmed  should_be=not_answered  safety=none

**answered_by**: TYP-47: "`...` after a type parameter's name makes it a **pack**: any number of types, none included (`[Cs...]` above `fn view() -> View[Cs...]:`). ... `Cs...` among type arguments spells the pack's types out (`View[Cs...]`), `values...` among a call's arguments passes the values on (`log(values...)`), and `values.len()` is their number, known when compiling."; TYP-47: "A pack is the last of its type parameters, and its values' parameter is the function's last positional one"; grammar III.3: "generic_arg   := type | expression | IDENT "=" type          -- const argument; associated-type binding
               | IDENT "..."                               -- a pack spelled out ([TYP-47])
tuple_type    := "(" ")" | "(" type "," type {"," type} [","] ")"        -- two or more elements ([TYP-27])
... fn_type_param := [param_word] type"; TYP-27: "Written as a type, `()` is `void`: a tuple type has two or more elements (ODR-071)."; TYP-19: "a generic may take any number of types (`[TYP-47]`, verdict 115)"; ECS-4: "A view takes any number of component types, with no limit (`reg.view[Position]()`, `reg.view[Position, Velocity, Health, Armor]()`; variadic generics, `[TYP-47]`), and any type is a component."; STD-26: "`enumerate(it, start=0)`, `zip(a, b, …)` (any number of iterables, `[TYP-47]`; stops at the shortest) and `reversed(x)` ... return lazy iterators"; JOB.md verdict 115: "Uses named to the owner: the ECS (`view`, `view_mut`, `exclude`, `all_of`, `any_of`), `zip` over any number of iterators, a tuple's interfaces written once for every size (Rust stops at 12), events and callbacks passing several values (`Event[int, String]` with handlers `fn(int, String)`)."

**program**: A pack can be spelled out only among type arguments (`View[Cs...]`); a tuple type and a function type take `type`s, so the library's own `View[Cs...]` and verdict 115's `Event[int, String]` cannot be written, and no rule says what the view's item is for one component or for none:

[Cs...]
struct View:
    reg: ref Registry
    # Iterator.Item must be the entity and one value per component:
    #   type Item = (Entity, Cs...)        -- not producible: tuple_type takes `type`s, not `IDENT "..."`
    # and for `View[Position]` it would be `(Entity, Position)`, for `View[]` `(Entity)`, a one-element tuple TYP-27 refuses

[Ts...]
class Event:
    handlers: Array[fn(Ts...)] = []        # not producible: fn_type_param takes a `type`
    fn fire(values: Ts...):
        for h in this.handlers:
            h(values...)

fn main():
    pairs = zip([1, 2])                    # one iterable: items `(1,)`, which does not exist, or `1`? unsaid
    hit = Event[int, String]()
    hit.handlers.push(owns fn(dmg: int, who: String): println(dmg, who))
    hit.fire(5, String("orc"))

- READER EDIT rule=grammar tuple_type
  - old: tuple_type    := "(" ")" | "(" type "," type {"," type} [","] ")"        -- two or more elements ([TYP-27])
  - new: tuple_type    := "(" ")" | "(" tuple_elem "," tuple_elem {"," tuple_elem} [","] ")" | "(" IDENT "..." ")"
                                                     -- two or more elements once a pack is spelled out ([TYP-27], [TYP-47])
tuple_elem    := type | IDENT "..."                        -- a pack spelled out
  - why: A tuple over a pack (`(Entity, Cs...)`, `(Ts...)`) is what the view's item and a tuple's interfaces "written once for every size" (verdict 115) need, and the grammar cannot produce it.

- READER EDIT rule=grammar fn_type_param
  - old: fn_type_param := [param_word] type
  - new: fn_type_param := [param_word] (type | IDENT "...")       -- a pack spelled out ([TYP-47])
  - why: Verdict 115 names `Event[int, String]` with handlers `fn(int, String)` as a use; a generic `Event` must write `fn(Ts...)`.

- READER EDIT rule=TYP-47
  - old: `Cs...` among type arguments spells the pack's types out (`View[Cs...]`),
  `values...` among a call's arguments passes the values on (`log(values...)`), and `values.len()` is
  their number, known when compiling.
  - new: `Cs...` among type arguments spells the pack's types out (`View[Cs...]`), and so does it among a tuple type's elements (`(Entity, Cs...)`) and a function type's parameters (`fn(Ts...)`, `Array[fn(Ts...)]`); a type written over a pack is formed once the pack is spelled out, so a tuple type that then has two or more elements is a tuple, one that has one is that type itself, and one that has none is `void` (`[TYP-27]`: there is no one-element tuple; `View[Position]`'s items are `(Entity, Position)`, `zip(xs)` of one iterable gives each item plainly, and a tuple's interfaces are written once from two elements up, `[A, B, Ts...]` above `extend (A, B, Ts...) implements Display:`). A `struct` or `class` takes a pack as a function does (`[Cs...]` above `struct View:`), its last type parameter. `values...` among a call's arguments passes the values on (`log(values...)`), and `values.len()` is their number, known when compiling.
  - why: Nothing said a type may take a pack or what a tuple over one is at the sizes TYP-27 excludes; the finder's A keeps the tuple rule that exists, adds no spelling, and gives exactly what verdict 115 asked for (one component type with no tuple around it). Nothing runs for any of it (`[MONO-1]`).

**remaining_doubt**: Verdict 115's "no tuple" for one type speaks of the brackets (`view[Position]`, not `view[(Position,)]`), not of the item type, so the one-and-none sizes are the lead's to rule; option B (a one-element tuple, a new spelling) is the alternative and contradicts TYP-27 and ODR-071.

**REFUTER reason**: Confirmed: TYP-47 "`Cs...` among type arguments spells the pack's types out (`View[Cs...]`)" and the grammar (`generic_arg := ... | IDENT "..."`; `tuple_type := "(" ")" | "(" type "," type {"," type} [","] ")"`; `fn_type_param := [param_word] type`) let a pack be spelled out only as a type argument, so the item type of the library's own `View[Cs...]` (ECS-4: "gives each item read-only (`for e, a, b in reg.view[A, B]():`)", a type written nowhere) and verdict 115's named uses ("a tuple's interfaces written once for every size (Rust stops at 12), events and callbacks passing several values (`Event[int, String]` with handlers `fn(int, String)`)") cannot be written; TYP-27 "a tuple type has two or more elements (ODR-071)" leaves `View[Position]`'s and `zip(xs)`'s item (STD-26: "`zip(a, b, …)` (any number of iterables ...)") undecided for one and none; a search for `(Entity, `, `Ts...)` or `Item = (` in the document finds nothing. The reader's edits are at the cause, add no spelling beyond C++'s (verdict 116: "A pack is written with C++'s dots"), keep TYP-27 (no one-element tuple, as the E0100 row says of `(e,)`), and run nothing (TYP-47: "Each use is made for its exact types ... nothing is looked up while the program runs"). Verdict 115's "no tuple" speaks of the brackets (`view[Position]`, not `view[(Position,)]`), so the one-and-none sizes are the lead's to rule, as the reader says; option B would contradict TYP-27 and the E0100 row.

## K70-raw-identifier-reserved  reader=answered  refuter=confirmed  should_be=answered  safety=none

**answered_by**: LEX-14: "A raw identifier `r#name` uses a keyword as a name (for imported C fields such as `type`). A keyword written where a name is expected, any keyword of the table below, is `E0005`: `type = 3` is "`type` is a keyword", with the help "write `r#type` to use the word as a name""; LEX-15: "**Reserved for a future version** (lexed as keywords; using one as a name is `E0005`, whose message names the reservation): `async`, `await`, `macro`. **Reserved and not words of Ember** (lexed as keywords so that no program uses them as names; any use is `E1010`, whose message gives the word to write): `owned` ... and `self` (inside a method its object is `this`, `[GRM-41]`)."; FFI identifier table (XVI.3): "| an identifier that is an Ember keyword | a raw identifier `r#type` (`[LEX-14]`) |"

**program**: 

**remaining_doubt**: The finder's (a) follows by composition: `owned`, `self`, `async`, `await` and `macro` are "lexed as keywords", `r#name` "uses a keyword as a name", and the importer maps "an identifier that is an Ember keyword" to `r#`, so `b.r#owned` is a raw identifier and only the bare word is `E1010` or `E0005`. The text has not changed on this point, and the two readings the checker saw differ only on whether a raw identifier is a "use" of the word; the sentence "uses a keyword as a name" answers that. A four-word clarification in LEX-14 ("any word the lexer reads as a keyword, the reserved ones of `[LEX-15]` included") would close the doubt; it is not a defect.

**REFUTER reason**: Decided by composition, as the reader says: LEX-14 "A raw identifier `r#name` uses a keyword as a name (for imported C fields such as `type`)"; LEX-15 "**Reserved and not words of Ember** (lexed as keywords so that no program uses them as names; any use is `E1010`, whose message gives the word to write): `owned` ... and `self`" and "**Reserved for a future version** (lexed as keywords; using one as a name is `E0005` ...): `async`, `await`, `macro`"; the FFI table (§XVI.3) "| an identifier that is an Ember keyword | a raw identifier `r#type` (`[LEX-14]`) |". The reserved words are lexed as keywords, a keyword is what `r#` turns into a name, and the importer maps every keyword identifier to `r#`, so a C field `owned` is reachable only as `b.r#owned`; the other reading (that `r#owned` is a "use" of the word) would make an imported field unreachable, which the FFI table's row rules out. Not a defect; the reader's four-word clarification would be harmless.

## K71-nested-tuple-access  reader=answered  refuter=confirmed  should_be=answered  safety=none

**answered_by**: LEX-18: "`1.` followed by an identifier character is a method call on `1`; `1.0` is a float. After `.` the lexer reads an integer, never a float: `t.0.1` is two field accesses."; grammar postfix: "| "[" expr_list "]" | "." IDENT | "." INT"

**program**: 

**remaining_doubt**: LEX-18 was changed in 0.9.20 to exactly the finder's (a): `edges[0].0.1` is `(edges[0].0).1`. Nothing remains.

**REFUTER reason**: LEX-18, changed in 0.9.20, decides it in words: "After `.` the lexer reads an integer, never a float: `t.0.1` is two field accesses", and the grammar's `postfix := ... | "." IDENT | "." INT`. So `edges[0].0.1` is `(edges[0].0).1`. Nothing remains.

# MISSED

## MISSED: A whole name as an element of a collection literal: handed over, viewed or refused?

**rules**: TYP-38, EXP-6, OWN-9. EXP-6 lists the uses that hand a whole name over: "used as a value (given with `=`, given to an `owns` parameter, returned, captured with `owns`)"; OWN-9 adds "`return a` hands `a` over with no word, and so does a constructor's argument, as `=` does: `w = Words(names=xs)` hands `xs` over"; TYP-38 says only "Every element converts to the element type by `[TYP-5]`" and "An element for a list of `owns` values is made with `owns`". A collection literal's element is in none of these lists, so for `xs = [a, b]` with `a: String` an implementer cannot say whether `a` ends (`E3040` after), whether `xs` is an `Array[ref String]` of views (which TYP-15 would allow a list to hold), or whether the line is refused.

**program**:
```
fn main():
    a = String("ann")
    b = String("bob")
    xs = [a, b]          # hands `a` and `b` over, or an Array[ref String] of views?
    println(a)           # E3040, or prints "ann"?
    xs.push(owns String("cy"))
```

- MISSED EDIT rule=OWN-9
  - old: `w =
  Words(names=xs)` hands `xs` over, `Point(x=a)` copies `a`.
  - new: `w = Words(names=xs)` hands `xs` over, `Point(x=a)` copies `a`; an element of a collection literal is given the same way (`xs = [a, b]` hands `a` and `b` over and a later use of `a` is `E3040`; `[copy of a, b]` keeps `a`; a `Copy` value is copied; a part, or a value reached through a name that does not own it, is `E3013` as elsewhere, `[OWN-11]`).
  - why: A literal element is the one value position that no rule names; a constructor's argument is the nearest rule and has the owner's reading (verdicts 74 and 85 write the making line that way), and the two helps (`copy of`, the `E3013` of a non-owning name) already exist.

## MISSED: `owns dyn I`: which of a required interface's implementations it has

**rules**: TYP-36, TYP-22. TYP-36's prose gives `owns dyn I` exactly two things, "`owns dyn I` when `I` requires `Clone`" and "`owns dyn I` compares when `I` requires `Eq`", and the table has no row for it, so `Ord`, `Hash`, `Display` and `Debug` are read from an omission; but TYP-22 reaches "a method of an interface that `I` requires ... through `dyn I` ... where it names `Self` nowhere but as `this`" and allows one `dyn` entry for a method "generic over a type parameter used only as the type of a parameter that views its value (a plain or `ref mut` parameter)", which `Display.fmt`, `Debug.fmt` and `Hash.hash[H: needs Hasher](ref mut h: H)` are, so by TYP-22 `owns dyn I` would have `Display`, `Debug` and `Hash` where `I` requires them, and by TYP-36's silence it would not. An implementer choosing `E2040` or acceptance for `Set[owns dyn Shape]` or `println(shape)` with `interface Shape: needs (Debug, Hash):` cannot tell which rule governs.

**program**:
```
interface Shape: needs (Debug, Hash, Ord):
    fn area() -> float

fn main():
    seen: Set[owns dyn Shape] = Set()      # needs Eq and Hash on `owns dyn Shape`: TYP-22 reaches `hash`, TYP-36 grants only Clone and Eq
    shapes: Array[owns dyn Shape] = []
    shapes.sort()                          # needs Ord: `cmp(other: Self)` is E2050 through dyn, so E2040 here, by inference only
    println(shapes)                        # Debug through dyn: TYP-22 yes, TYP-36 silent
```

- MISSED EDIT rule=TYP-36
  - old: `owns dyn I` compares when `I` requires `Eq`
  (`interface Shape: needs Eq:`, a method-table entry as for `Clone`): values of two different kinds
  are never equal, and two of one kind compare their contents; that entry is the one way `Eq` is
  reached through `dyn` (`[TYP-22]`).
  - new: `owns dyn I` compares when `I` requires `Eq` (`interface Shape: needs Eq:`, a method-table entry as for `Clone`): values of two different kinds are never equal, and two of one kind compare their contents; that entry is the one way `Eq` is reached through `dyn` (`[TYP-22]`). Of the other interfaces `I` requires, `owns dyn I` (and `shares dyn I`, `shares mut dyn I`, `dyn I` where a view may stand) has those whose methods `[TYP-22]` reaches through `dyn`, `Display`, `Debug` and `Hash` (each method names `Self` only as `this`, and `hash`'s hasher is a parameter that views its value, so it gets one entry), and never `Ord` (`cmp` takes `other: Self`): a bound needing `Ord` on it is `E2040`, whose help is `sort_by` with the kinds tested first.
  - why: Two rules answer the question in different ways by silence; one sentence makes TYP-36 say what TYP-22's mechanism gives, so the `—` of a missing row is no longer the only evidence.

## MISSED: A pack spelled out inside a type built on each member (`Option[Cs]...`) is not producible

**rules**: TYP-47, grammar generic_arg. Verdict 116 chose "C++'s spelling", and TYP-47 says "`Cs...` among type arguments spells the pack's types out (`View[Cs...]`)"; the grammar's `generic_arg := ... | IDENT "..."` spells out only a bare pack name, so a type formed over each member, C++'s pattern expansion (`Option[Cs]...`, `Span[Cs]...`), is not producible, and the text does not say whether that is meant. The ECS's own `View` would want it for a method giving each component as an `Option` (`try_get -> (Option[Cs]...)`) and a library `zip` of spans (`Span[Ts]...`) cannot be typed. Not a safety matter; an implementer cannot tell whether a refusal is the design or a gap.

**program**:
```
[Cs...]
struct View:
    reg: ref Registry
    fn try_get(e: Entity) -> (Option[Cs]...):     # `Option[Cs]...` is no generic_arg, tuple element or pattern the grammar has
        pass
```

- MISSED EDIT rule=TYP-47
  - old: 
  - new: `...` is written after a pack's name only (`Cs...`, `values...`); a type formed over each type of the pack (`Option[Cs]...`, C++'s pattern expansion) is not Ember, and is `E0100`, whose help is a second pack or a tuple of the spelled-out pack (`(Option[Cs]...)` is not it either; write the type the use needs, `[Os...]` above the function, with `Os` the options).
  - why: The grammar refuses it today; saying so, with its reason, keeps an implementer from reading the owner's "C++'s spelling" as C++'s whole pack grammar. The lead may instead rule the expansion in; then `generic_arg`, `tuple_elem` and `fn_type_param` take `type "..."` where the type names exactly one pack, and TYP-47 says it is written out once per member, nothing running for it (`[MONO-1]`).
