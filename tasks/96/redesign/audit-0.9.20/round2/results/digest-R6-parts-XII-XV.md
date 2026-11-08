# R6-parts-XII-XV

could_not_cover: All ten clusters were read against the current text. Not checked beyond my clusters: verdict 165 against [THR-1] (its second bullet names only `Send`, "everything reachable from it is `Send` (`[THR-8]`)", while the verdict refuses a `thread_shared` value whose type is not `ThreadShared` where it is made; [THR-9]'s "`thread_shared T` for a `T` that is `Send` and `ThreadShared`" may or may not be read as the refusal at that line). The 0.9.15 std sources were used only to confirm names and the `rsqrt`/`sin` facts; the specification was the authority throughout.

## K13-weakref-to-static  reader=not_answered  refuter=narrowed  should_be=not_answered  safety=hole_in_current_text

**answered_by**: STA-1: "`static NAME: T = e` is one value per program with a stable address, and it never changes; shared state that changes goes through an `Atomic`, `Mutex`, `RwLock` or `Once` in it. [...] `T` MUST be `ThreadShared` (`E7002`)."; THR-9: "Structural types are `ThreadShared` when every component is."; STA-3: "A static whose initialiser can be evaluated at compile time is placed in the image with no initialisation code, except a value that owns heap memory (an `Array`, `String`, `Map`, `Set`, an `owns T`, a class object) in a `static mut`, or in a `static` whose type is one a plain name may change"; CT-5: "in a `static` (not `static mut`) whose type is not one a plain name may change, the value is materialised in read-only memory; statics are never dropped. A type a plain name may change is one with, on any path to a field, a `Mutex`, `RwLock`, `Cell`, `Atomic`, `Once`, a channel end, a `guarded by` field, or a class with a `@cached` method"; WK-11: "`weakref T` links a value on one thread: a class object with one owner or with `shares` or `shares mut` owners [...] Every `weakref` link holds one weak count on its target: making one adds one [...] That count is a number in the target's header that no name reads, so making or ending a link is not a change of the target: any name may make one"; OBJ-1: "a type carries a weak count when **it can be named from outside its package** [...] **or when a `weakref` type in the package names it** (written, inferred at a `weakref x`, or in a generic instance the package makes, `[MONO-1]`)"; OWN-13: "and `weakref T` otherwise (a class object with one owner, a `shares` or `shares mut` owner, a `weakref T` link, a name opened from one, a view of any of these)"; THR-2: "**A `weakref T` link names only an object reached through this thread's own owners** (as Rust's `Rc` never sits inside an `Arc`): `weakref x` where the place `x` is reached through a `thread_shared` value is `E7015`, at the line or at a call whose callee makes such a link to its parameter"; THR-2: "**Making, ending or opening a `weakref` link touches its target's header** (its weak count, and the test that it still lives): in a scoped task, a job, or any function that may run on another thread than its target's owner, it is allowed only through a `weakref thread_shared T`"; THR-13: "a module variable: a `static` never changes (`[STA-1]`), and a `static mut` is reached only by the main thread and the tasks and jobs of its scopes, or is `guarded by` a lock (`[THR-20]`)"; THR-20: "A `static` of a type that changes only through itself (`Lock`, `Atomic`, `Mutex`, `RwLock`, a `Sender`; a `Receiver` is not `ThreadShared`, `E7002`, `[STA-1]`) is usable from any thread."; OBJ-1: "Which thread made a value is read from the header of the memory chunk its block came from, never from the value [...] A `thread_shared` value made `using` an instance (`[ALC-1]`) lies in memory the runtime did not lay out, so it is marked handed over at the line that makes it (`[RC-4]` point 4) and no thread asks which thread made it."

**program**: class Node:
    x: int = 0

static ORIGIN: Node = Node(x=0)        # Node is ThreadShared (one int field), so STA-1 accepts it;
                                        # STA-3/CT-5 put the object, header included, in read-only memory;
                                        # Node carries a weak count (OBJ-1: a weakref type below names it)

fn track():
    keep = weakref ORIGIN               # WK-11: any name may make one; OWN-13: a `weakref Node` (no
                                        # thread_shared word on the path); THR-2's E7015 test asks only
                                        # whether the path crosses a `thread_shared` value: it does not
    if keep exists as n:
        println(n.x)

fn main():
    h = thread.spawn(owns fn(): track())   # the function captures nothing one-thread: accepted
    track()
    h.join()

What the accepted program does: `keep = weakref ORIGIN` adds one to the weak count in ORIGIN's header, which CT-5 placed in read-only memory (a fault, in a program with no `unsafe`; already on one thread, in `main` alone), and the two threads add to that plain count at once (a data race, `[PHIL-10]`). THR-2's bold principle ("reached through this thread's own owners") would refuse it, its operative test ("reached through a `thread_shared` value") accepts it, and no rule says which thread owns a static's object. The same cause breaks `static ORIGIN: thread_shared Node = thread_shared Node(x=0)` (STA-1 accepts: THR-9 makes `thread_shared Node` ThreadShared): `fn() thread_shared ORIGIN:` adds to its atomic owner count, and `weakref ORIGIN` (a `weakref thread_shared Node`, OWN-13) to its weak count, both in the read-only image, and OBJ-1 reads its making thread from a chunk header the image does not have.

- READER EDIT rule=THR-2
  - old: **A `weakref T` link names only an object reached through this
  thread's own owners** (as Rust's `Rc` never sits inside an `Arc`): `weakref x` where the place `x`
  is reached through a `thread_shared` value is `E7015`,
  - new: **A `weakref T` link names only an object reached through this thread's own owners** (as Rust's `Rc` never sits inside an `Arc`): `weakref x` where the place `x` is reached through a value every thread may reach, a `thread_shared` value, a `static` (not an unguarded `static mut`, which the main thread and its scoped tasks alone reach, `[THR-20]`), a `guarded by` `static mut` or a `comptime(e)` result (`[STA-1]`, `[CT-5]`), is `E7015` ("`ORIGIN` is a `static`, which every thread may read, and `weakref ORIGIN` would count its links on this thread alone; read `ORIGIN` by its name, or share it between threads: write `static ORIGIN: thread_shared Node = thread_shared Node(x=0)` in place of line 4, after which `weakref ORIGIN` is a `weakref thread_shared Node`"),
  - why: The operative test named only a `thread_shared` path, while a `static`, a guarded `static mut` and a `comptime(e)` result are reached by every thread too; a plain weak count written from several threads is the race the bold sentence exists to prevent. The property stated (a place every thread may reach) covers every such route, and the link that works is the one whose counts are atomic.

- READER EDIT rule=CT-5
  - old:   * in a `static` (not `static mut`) whose type is not one a plain name may change, the value is
    materialised in read-only memory; statics are never dropped.
  - new:   * in a `static` (not `static mut`) whose type is not one a plain name may change, the value is materialised in read-only memory, except a value that is, or owns, a `thread_shared` value, whose counts another owner (`fn() thread_shared st:`) or a `weakref thread_shared T` link writes with atomics: that value is materialised in the image in memory that may be written, with no initialisation code, each `thread_shared` value in it marked handed over (`[RC-4]` point 4, as a value made `using` an instance is), so that no thread asks which thread made it; its own count never reaches zero, since the static holds one owner for the life of the program. Statics are never dropped. No other header in constant data is ever written: a `weakref T` link to a value a `static` or a `comptime(e)` result reaches is `E7015` (`[THR-2]`), and `shares` of one is `E2122` (`[RC-1]`). A `comptime(e)` whose value is, or owns, a `thread_shared` value is materialised the same way;
  - why: The placement rule put every header of a counted value in read-only memory; with `weakref T` refused on these paths, the only headers a line writes in image data are a `thread_shared` value's atomic counts, so only those are placed writable, at no run-time cost (no first-access test), and OBJ-1's chunk-header lookup is answered as it is for a `using` value.

**remaining_doubt**: Whether the lead prefers the cluster's option A (allow `weakref` to a plain static's object, kept writable, owned by the main thread): it cannot be made safe without knowing which thread runs the line, which THR-2's whole design avoids, so the refusal with the `thread_shared` static as the line to write is the consistent answer; verdict 166 (a scoped task makes no `weakref T` link) is untouched. The `thread_shared`-static half of the edit (writable placement, handed-over mark) is a fact the text must state somewhere; CT-5 is where placement is decided, STA-3 refers to it. Whether `thread_shared Node(x=0)` is accepted as a compile-time initialiser at all (Alloc is allowed by CT-1) the text already implies; if the lead would rather refuse `thread_shared` in a `static`, STA-1 is the place, and the K13 program then has no fixed marker to link to.

**REFUTER reason**: Confirmed as a hole, with the reader's two edits kept and two consistency edits added. The program is accepted: `[STA-1]` "`T` MUST be `ThreadShared` (`E7002`)" and the §IV.8 marker table's `ThreadShared` row "every component is `ThreadShared`" admit `class Node` with one `int` field (its `Send` row says outright "a class object is when its fields are"). `[STA-3]` places it: "A static whose initialiser can be evaluated at compile time is placed in the image with no initialisation code, except a value that owns heap memory (…, a class object) in a `static mut`, or in a `static` whose type is one a plain name may change" — a plain `static` of a class type is neither exception, so `[CT-5]` applies: "in a `static` (not `static mut`) whose type is not one a plain name may change, the value is materialised in read-only memory", and a weak count is not on `[CT-5]`'s list of what a plain name may change ("a `Mutex`, `RwLock`, `Cell`, `Atomic`, `Once`, a channel end, a `guarded by` field, or a class with a `@cached` method"). `ORIGIN`'s header carries a weak count (`[OBJ-1]`: "or when a `weakref` type in the package names it (written, inferred at a `weakref x`…)"). `weakref ORIGIN` is accepted: `[WK-11]` "`weakref T` links a value on one thread: a class object with one owner" and "any name may make one"; `[WK-16]` refuses only a part, an `owns` value or a new value; `[THR-2]`'s operative test is "`weakref x` where the place `x` is reached through a `thread_shared` value is `E7015`", and a `static` is not one. `[RC-1]` already refuses `shares ORIGIN` ("written before a new value, a local of this function that owns its value … or a name that is already a `shares` or `shares mut` owner"), so the weak count is the one count a safe line writes into image data: `[WK-11]` "making one adds one" — a write into memory `[CT-5]` made read-only, breaking `[CT-5]`'s "no run-time code ever grows, changes or frees constant data" in `main` alone, and from two threads a plain-add race (`[THR-2]`: "every object that is not `thread_shared` counts its `weakref` links with plain adds"), which `[PHIL-10]` promises cannot happen. No verdict decides it: verdict 166 refuses a scoped task's link and is untouched; the fix follows `[THR-2]`'s own principle ("names only an object reached through this thread's own owners": a `static` is owned by no thread). The `thread_shared` static half is also real: `[OBJ-1]` "Which thread made a value is read from the header of the memory chunk its block came from", and an image-placed value has no chunk; the reader's writable placement with the handed-over mark reuses the mechanism `[RC-4]` point 4 already has ("every count operation on a `thread_shared` value … first tests the handed-over mark, one load and one branch"), so no new run-time cost and no new §X.4 row. Narrowed only for completeness: the `E7015` registry row still says "reached through a `thread_shared` value" and `[OBJ-1]`'s handed-over sentence names only the `using` case, so both are edited to claim what the rules will say; and the CT-5 edit states where the static's one owner is counted (the away count), which an implementer otherwise has to infer.

- BETTER EDIT rule=THR-2
  - old: **A `weakref T` link names only an object reached through this
  thread's own owners** (as Rust's `Rc` never sits inside an `Arc`): `weakref x` where the place `x`
  is reached through a `thread_shared` value is `E7015`,
  - new: **A `weakref T` link names only an object reached through this thread's own owners** (as Rust's `Rc` never sits inside an `Arc`): `weakref x` where the place `x` is reached through a value every thread may reach, a `thread_shared` value, a `static` (not an unguarded `static mut`, which the main thread and its scoped tasks alone reach, `[THR-20]`), a `guarded by` `static mut` or a `comptime(e)` result (`[STA-1]`, `[CT-5]`), is `E7015` ("`ORIGIN` is a `static`, which every thread may read, and `weakref ORIGIN` would count its links on this thread alone; read `ORIGIN` by its name, or share it between threads: write `static ORIGIN: thread_shared Node = thread_shared Node(x=0)` in place of line 4, after which `weakref ORIGIN` is a `weakref thread_shared Node`"),
  - why: The reader's edit, kept: the operative test named only a `thread_shared` path while a `static`, a guarded `static mut` and a `comptime(e)` result are reached by every thread too; the property (a place every thread may reach) covers every route, and the link that works is the one whose counts are atomic.

- BETTER EDIT rule=CT-5
  - old:   * in a `static` (not `static mut`) whose type is not one a plain name may change, the value is
    materialised in read-only memory; statics are never dropped.
  - new:   * in a `static` (not `static mut`) whose type is not one a plain name may change, the value is materialised in read-only memory, except a value that is, or owns, a `thread_shared` value, whose counts another owner (`fn() thread_shared st:`) or a `weakref thread_shared T` link writes with atomics: that value is materialised in the image in memory that may be written, with no initialisation code, each `thread_shared` value in it marked handed over with its one owner, the static's, counted in its away count (`[RC-4]` point 4, as a value made `using` an instance is), so that no thread asks which thread made it and every count operation on it finds the mark first; its count never reaches zero, since the static holds that owner for the life of the program. Statics are never dropped. No other header in constant data is ever written: a `weakref T` link to a value a `static` or a `comptime(e)` result reaches is `E7015` (`[THR-2]`), and `shares` of one is `E2122` (`[RC-1]`). A `comptime(e)` whose value is, or owns, a `thread_shared` value is materialised the same way;
  - why: The reader's edit with the away count named: the placement rule put every header of a counted value in read-only memory; with `weakref T` refused on these paths, the only header a line writes in image data is a `thread_shared` value's atomic counts, so only those are placed writable, at no run-time cost; `[RC-4]` point 4's handed-over form makes every thread count in the away count, which is where the static's owner must be.

- BETTER EDIT rule=registry row E7015
  - old: a `weakref T` link made to a value reached through a `thread_shared` value, at the line or at a call by the callee's summary;
  - new: a `weakref T` link made to a value reached through a value every thread may reach (a `thread_shared` value, a `static`, a `guarded by` `static mut`, a `comptime(e)` result), at the line or at a call by the callee's summary;
  - why: The row must claim what `[THR-2]` says once the test is widened (one code, one defining rule).

- BETTER EDIT rule=OBJ-1
  - old: so it is marked handed over at the line that makes it (`[RC-4]` point 4)
  and no thread asks which thread made it.
  - new: so it is marked handed over at the line that makes it (`[RC-4]` point 4)
  and no thread asks which thread made it; so is a `thread_shared` value a `static` or a `comptime(e)` holds, placed in the image when compiling (`[CT-5]`).
  - why: `[OBJ-1]` is where the chunk-header lookup is stated; an image-placed value has no chunk, and the sentence that exempts the `using` case must name the second exemption, or the lookup reads memory before the static.

## K20-anyerror-send  reader=answered_differently  refuter=confirmed  should_be=answered_differently  safety=none

**answered_by**: ERR-8: "**`AnyError`** is the prelude's "any error" type: a `dyn Error` held alone (`owns dyn Error`) plus an optional chain of context messages. It implements `From[E]` for every `E: needs Error`, so `?` converts any error into it"; §IV.8 (ERR-3): "interface Error: needs (Debug, Display):
    fn source() -> Option[ref dyn Error]"; THR-8: "Structs, enums, tuples, fixed arrays, collections and class objects are `Send` when every component is; an `owns T` when `T` is; [...] `dyn I` when `I` requires it (`interface I: needs Send:`)."; THR-10: "`[R]` above `pub fn spawn(owns f: thread_shared once fn() -> R) -> JoinHandle[R]:` [...] and `R` MUST be `Send` (`E7015`, `E7008`)."; ERR-9: "The error parameter defaults to `AnyError` (§XIII.2), so `fn load(path: StrView) -> Result[Texture]` is a function that can fail with any error."

**program**: 

- READER EDIT rule=ERR-8
  - old: 
  - new: `AnyError` is neither `Send` nor `ThreadShared`: the error it holds is a `dyn Error`, and `Error` does not require `Send` (`[THR-8]`), so an error that holds a one-thread owner may be inside it. A thread's result (`[THR-10]`), a channel's element (`[THR-15]`) or a `thread_shared` value whose type holds an `AnyError`, a `Result[T]` with its default error included (`[ERR-9]`), is `E7015` ("`load` fails with `AnyError`, which may hold an error that stays on one thread, and `thread.spawn`'s result goes to the thread that joins it; name the error type: write `fn load(path: String) -> Result[int, io.Error]` in place of line 4, and convert after the join: `n = h.join()?`").
  - why: The rules decide the refusal, but by three rules in three Parts and with no message; the cluster's critic noted that an ordinary program is refused without the text saying so. The sentence states the property where the type is defined and gives the line to write.

**remaining_doubt**: The current answer is the cluster's option B (refuse), safe, and against the checker's recommendation A (make `AnyError` `Send` and `ThreadShared`, `From[E]` only for `E: needs Error + Send`, so `?` of an error holding a `shares` field into `AnyError` is `E2040` at the `?` even in a one-thread program, as Rust's `anyhow::Error` requires `Send + Sync`). No verdict decides it; A costs nothing while running and makes `thread.spawn(owns fn() => load(...))` with `-> Result[int]` compile, which is ordinary code, so the lead may prefer A: then ERR-8's `From[E]` bound changes and THR-8 lists `AnyError` as `Send`. `std/src/core.em:92` (`error: Box[dyn Error]`) is consistent with B today.

**REFUTER reason**: The rules decide it, and safely: `[ERR-8]` "a `dyn Error` held alone (`owns dyn Error`)"; `[THR-8]` "an `owns T` when `T` is; … `dyn I` when `I` requires it (`interface I: needs Send:`)", and `[ERR-3]`'s `Error` is "`Display + Debug` with `fn source() -> Option[ref dyn Error]`", requiring no `Send`; so `AnyError` is not `Send`, and `[THR-10]` "`R` MUST be `Send` (`E7015`, `E7008`)" refuses `thread.spawn(owns fn() => load(...))` with `-> Result[int]` (`[ERR-9]`: "The error parameter defaults to `AnyError`"), as `[THR-15]` refuses a channel of it ("`Sender[T]` and `Receiver[T]` are `Send` … exactly when `T` is `Send`") and `[THR-1]` a `thread_shared` value holding one ("everything reachable from it is `Send`"). The refusal is option B (refuse), safe. The reader's edit states the derived fact at the type's definition with a message that gives the line to write; it opens no hole (it refuses nothing the rules accept), contradicts no rule, adds no cost, and no verdict (164 to 166) touches `AnyError`. Whether to move to option A (`AnyError` `Send` and `ThreadShared`, `From[E]` for `E: needs Error + Send`, as `anyhow::Error` requires `Send + Sync`) is a change of what compiles, which the owner reserved ("What a program does, and whether it compiles, stays the owner's", JOB.md); the edit as written does not need it.

## K29-text-view-in-list  reader=answered_differently  refuter=narrowed  should_be=not_answered  safety=none

**answered_by**: STD-8: "`x in c` requires `c: Contains[typeof(x)]` (`E2226` otherwise, whose help is `c.iter().any(owns fn(e) => e == x)`). `std` implements it for `Set` and `Map` (by key); for `Array`, `Span`, `MutSpan` and `[T; N]` with `T: needs Eq`, as a linear scan"; STD-8b: "`x not in c` is `not c.contains(x)`, with each operand evaluated once."; §IV.8: "interface Contains[T]:
    fn contains(item: T) -> bool                                    # x in c"; STD-12: "Lookup methods, `m[k]` and `k in m` accept any key type `Q` with `Q: needs AsKey[K]`: `K` itself, and `StrView` for `K = String`. [...] std implements `AsKey[String]` and `ToKey[String]` for `StrView`."; LEX-20: "A string literal has type `StrView` with the static region: a view of text written in the program. At a site expecting `String` it produces a `String` (`[TXT-9]`)."; TXT-9: "Wherever a `String` is expected — an initialiser, an argument, a field, a collection element, a return value — a string literal produces a new `String` holding its text. The allocation happens at the literal and is reported there (`Alloc`)"; TYP-5: "4. string literal to `String` (`[TXT-9]`), and `String` to `StrView` (a read-only view, `[SPN-1]`);"; §IV.8: "interface Eq:
    fn eq(other: Self) -> bool                          # ==, !="

**program**: 

- READER EDIT rule=STD-8
  - old: `x in c` requires `c: Contains[typeof(x)]`
  (`E2226` otherwise, whose help is `c.iter().any(owns fn(e) => e == x)`). `std` implements it for
  `Set` and `Map` (by key); for `Array`, `Span`, `MutSpan` and `[T; N]` with `T: needs Eq`, as a
  linear scan;
  - new: `x in c` is `c.contains(x)` (`[STD-8b]`), so `c` MUST implement `Contains[Q]` for the type of `x` (`E2226` otherwise, whose help is `c.iter().any(owns fn(e) => x == e)`); a literal needle is a `StrView` (`[LEX-20]`) where an implementation takes one, and makes a `String` (`[TXT-9]`, one allocation) only where none does. `std` implements it for `Set` and `Map` (by key, for every `Q: needs AsKey[K]`, `[STD-12]`); for `Array`, `Span`, `MutSpan` and `[T; N]` with `T: needs Eq`, as a linear scan, and for those of `String` with a `StrView` needle as well, each element read as a view (`[TYP-5]`), so a text view is looked for in any collection of `String`s alike and `who.trim() in names` or `"bob" in names` makes no `String`;
  - why: Two things were left to the implementer: how a literal needle is typed (LEX-20 says `StrView`, TXT-9 says `String` at an argument, and `contains(item: T)` makes it an argument), and why a set of `String`s takes a view needle (`[STD-12]`) while a list of them refuses it; the edit states both as properties. The `StrView` implementations for the `String` collections mirror the one `[STD-12]` already makes for `StrView` keys.

**remaining_doubt**: The current text decides the checker's option B (`who.trim() in names` is `E2226`; a literal needle, read as `contains`'s argument, makes a `String`), which is safe; the edit adopts the recommended A because no verdict decides it and the owner's rule that every value a type supports works in every construct points at it; the lead may keep B, in which case only the literal-typing sentence of the edit is needed. Two small facts beside it: the registered help `e == x` does not compile for a `StrView` needle against `String` elements, since `Eq.eq(other: Self)` admits the `String` → `StrView` coercion only on the right (`x == e` does); and TXT-9's "an argument" and LEX-20's "has type `StrView`" need the sentence above to agree where both apply.

**REFUTER reason**: Two things are in the cluster. `who.trim() in names` with `names: Array[String]` is decided and safe: `[STD-8]` "`x in c` requires `c: Contains[typeof(x)]` (`E2226` otherwise …). `std` implements it for … `Array`, `Span`, `MutSpan` and `[T; N]` with `T: needs Eq`, as a linear scan", so `Array[String]` is `Contains[String]` only and a `StrView` needle is `E2226` (`[TXT-9]`: "A `StrView` *value* never converts implicitly"). But `"bob" in names` is not decided: `[LEX-20]` "A string literal has type `StrView` … At a site expecting `String` it produces a `String`"; `[STD-8]`'s `typeof(x)` is then `StrView` (`E2226`), while `[STD-8b]` "`x not in c` is `not c.contains(x)`" and §IV.8's `fn contains(item: T) -> bool  # x in c` make the needle an argument, where `[TYP-5]` rule 4 ("string literal to `String`") makes it a `String` (accepted, one allocation). An implementer cannot pick; the second half of the cluster is therefore `not_answered`, not `answered_differently`. The registered help is also wrong for a view needle: `[§IV.8]` `interface Eq: fn eq(other: Self) -> bool`, so `e == x` with `e: ref String` and `x: StrView` has no coercion (`[TXT-9]`), while `x == e` has (`[TYP-5]` rule 4, `String` to `StrView`). The reader's edit stands: its first sentence decides the literal (a `StrView` where an implementation takes one, a `String` only where none does, which is `[TYP-5]`'s coercion applied only where needed), its help compiles for both needle types, and its `Contains[StrView]` for collections of `String` mirrors `[STD-12]` ("std implements `AsKey[String]` and `ToKey[String]` for `StrView`") at no run-time cost (a linear scan comparing views) and with no verdict against it; the lead may hold that half back and keep only the literal sentence and the help, which is the smallest change at the cause.

## K49-ser-skip-field-value  reader=not_answered  refuter=narrowed  should_be=not_answered  safety=none

**answered_by**: SER-1: "Field attributes: `@ser(skip)`, `@ser(rename="…")`, `@ser(default)`, `@ser(since=N)`."; SER-2: "Deserialising never produces an invalid value: a range-typed field is checked (`[RNG-10a]`), an unknown enum tag is an error, and a missing field without `@ser(default)` or `@ser(since=…)` is an error."; §XIV.2 (std.ser paragraph): "A field missing from the data takes its type's `Default` under `@ser(default)` or `@ser(since=N)`."; DRV-1: "| `Default` | each field's declared default, else its type's `Default`; for an enum, the variant marked `@default` |"; DRV-1: "A derive whose requirement a field does not meet is `E2080`, naming the field."

**program**: Implementer's question, not decidable from the text: after `SaveGame.deserialize(ref mut r)` for

@derive(Serialize, Deserialize)
struct SaveGame:
    level: int
    @ser(skip)
    cache: Map[String, int]     # never written: by SER-2 it is "a missing field without @ser(default)", so an
                                # error on every read; or it takes Map's Default; nothing says which
    @ser(skip)
    frames: int = 0             # its declared default, int's Default, or the same error?

and whether `@ser(skip)` on a field whose type has no `Default` and no declared default (`captain: shares Player`) compiles. A derive implementer must write `deserialize` one way; the data never holds the field, so the SER-2 reading makes every struct with a skipped field unreadable unless `@ser(default)` is also written, which SER-1 does not say.

- READER EDIT rule=SER-1
  - old: Field attributes: `@ser(skip)`, `@ser(rename="…")`,
  `@ser(default)`, `@ser(since=N)`.
  - new: Field attributes: `@ser(skip)`, `@ser(rename="…")`, `@ser(default)`, `@ser(since=N)`. A `@ser(skip)` field is written to no format and read from none: `deserialize` gives it what `@derive(Default)` gives a field (`[DRV-1]`: its declared default, else its type's `Default`), so it is never a missing field (`[SER-2]`'s error is for a field the type writes and the data lacks); a skipped field with no declared default whose type has no `Default` is `E2080`, naming the field ("`captain` is `@ser(skip)`, so reading a `SaveGame` back must give it a value, and `shares Player` has no `Default`; give the field a default (`captain: shares Player = …`) or write it to the data: remove `@ser(skip)`").
  - why: The attribute was named with no meaning for the reading side; the edit reuses the Default derive's rule, so a skipped field needs no second notion of default, and refuses only a field that truly has no value, at compile time.

**remaining_doubt**: The §XIV.2 paragraph says a missing field under `@ser(default)` or `@ser(since=N)` takes "its type's `Default`", while DRV-1's `Default` row takes the declared default first; the lead may want the paragraph to say "what `@derive(Default)` gives it" so the two readings agree. The rest of the finder's question (link fields, `owns dyn I`) the checker narrowed away: DRV-1's `E2080` sentence refuses a field whose type has no `Serialize`.

**REFUTER reason**: Confirmed as an implementer's undecidable line: `[SER-1]` "Field attributes: `@ser(skip)`, `@ser(rename=\"…\")`, `@ser(default)`, `@ser(since=N)`" gives `skip` no meaning on the reading side; `[SER-2]` "a missing field without `@ser(default)` or `@ser(since=…)` is an error" would make every type with a skipped field unreadable, and §XIV.2 "A field missing from the data takes its type's `Default` under `@ser(default)` or `@ser(since=N)`" says nothing of `skip`. The reader's edit reuses `[DRV-1]`'s `Default` row ("each field's declared default, else its type's `Default`") and refuses at compile time with `E2080` ("A derive whose requirement a field does not meet is `E2080`, naming the field"), the right place and no run-time cost. Narrowed twice: the message's second help ("remove `@ser(skip)`") leads into `E2080` again, since `shares Player` has no `Serialize` (the reader's own note), so only the default is offered; and the §XIV.2 paragraph must agree with the rule it now sits beside, or a field with both a declared default and `@ser(default)` reads two ways (the type's `Default` by §XIV.2, the declared one by `[DRV-1]`).

- BETTER EDIT rule=SER-1
  - old: Field attributes: `@ser(skip)`, `@ser(rename="…")`,
  `@ser(default)`, `@ser(since=N)`.
  - new: Field attributes: `@ser(skip)`, `@ser(rename="…")`, `@ser(default)`, `@ser(since=N)`. A `@ser(skip)` field is written to no format and read from none: `deserialize` gives it what `@derive(Default)` gives a field (`[DRV-1]`: its declared default, else its type's `Default`), so it is never a missing field (`[SER-2]`'s error is for a field the type writes and the data lacks); a skipped field with no declared default whose type has no `Default` is `E2080`, naming the field ("`captain` is `@ser(skip)`, so reading a `SaveGame` back must give it a value, and `shares Player` has no `Default`; give the field a default: `captain: shares Player = shares Player()`").
  - why: The attribute had no meaning on the reading side; the edit reuses the `Default` derive's rule, so a skipped field needs no second notion of default, refuses only a field that truly has no value, at compile time, and offers the one help that compiles.

- BETTER EDIT rule=§XIV.2 (std.ser paragraph)
  - old: A field missing from the data takes its type's `Default` under
`@ser(default)` or `@ser(since=N)`.
  - new: A field missing from the data takes, under
`@ser(default)` or `@ser(since=N)`, what `@derive(Default)` gives it (`[DRV-1]`: its declared default, else its type's `Default`), as a `@ser(skip)` field does (`[SER-1]`).
  - why: The paragraph said "its type's `Default`" while `[DRV-1]` takes the declared default first; one rule for every field that is not read from the data.

## K55-nondet-math-functions  reader=not_answered  refuter=narrowed  should_be=not_answered  safety=hole_in_current_text

**answered_by**: DET-2: "`Nondet` is introduced by exactly: floating-point contraction, reassociation or any `@fastmath` relaxation; a transcendental function from the platform library instead of `std.math.det`; observing a pointer or an object's address as an integer"; §X effect table: "| `Nondet` | a result that may differ between runs or machines (`[DET-2]`) |"; STD-27: "A method is the C library's function of its name for `f64` and its `f` form for `f32` (`ln` is C's `log`, `round` its `nearbyint`, `round_half_away` its `round`, `mul_add` its `fma`); the transcendental ones are `[DET-2]`'s platform functions."; STD-21: "`std.math` provides `PI`, `TAU`, `E`; `sin`, `cos`, `tan`, `asin`, `acos`, `atan`, `atan2`, `sinh`, `cosh`, `tanh`, `exp`, `exp2`, `ln`, `log2`, `log10`, `pow`, `sqrt`, `rsqrt`, `cbrt`, `hypot`, `lerp`, `smoothstep`"; DET-4: "`sqrt` is IEEE 754's, correctly rounded."; STD-3: "`math.fma(a, b, c)` (and `mul_add`) is a fused multiply-add with a single rounding, on every target"; TYP-9: "Floating point is strict IEEE 754: no reassociation, no contraction of `a * b + c` into a fused operation, no assumption that values are finite, unless a function is `@fastmath`"; SIMD-4: "approximations named `_approx` (`rsqrt_approx`, `rcp_approx`) are provided."; Part XV preamble: "`std/**/*.em` holds the exact signatures and becomes normative for anything this Part leaves open."

**program**: import std.math
from std.simd import f32x4

@deterministic
fn boost(speed: f64) -> f64:
    return math.cbrt(speed)            # algebraic, so not "transcendental": DET-2's exact list admits it;
                                       # it is the platform's `cbrt`: 3.0 under MSVC, 3.0000000000000004 under
                                       # glibc for 27.0 (the owner's own note), so it "may differ between
                                       # machines": the effect table's definition refuses it

@deterministic
fn inv_lengths(v: f32x4) -> f32x4:
    return v.rsqrt_approx()            # in neither of DET-2's float entries; its bits are the CPU maker's

The implementer cannot decide whether `boost` and `inv_lengths` are `E4070`: DET-2 ("exactly", "transcendental") says no, the effect table's definition of `Nondet` says yes, and `hypot` is the same case. (`(target - vel).normalize()` is decided: `sqrt` is correctly rounded on every target and `/` is IEEE, so the finder's `steer` compiles.)

- READER EDIT rule=DET-2
  - old: a
  transcendental function from the platform library instead of `std.math.det`;
  - new: a floating-point function the platform's C library computes whose result IEEE 754 does not fix to one correctly rounded answer (`[STD-27]`'s methods `sin` through `hypot`, `cbrt` and `hypot` among them; not `sqrt`, `fma`/`mul_add`, `abs`, `copysign`, `floor`, `ceil`, `trunc`, `round`, `round_half_away` or `fract`, which every target computes alike, nor `rsqrt`, `lerp` and `smoothstep`, which `std.math` builds from those), where `std.math.det`'s is not used; a SIMD `_approx` operation (`[SIMD-4]`), whose bits are the processor's own;
  - why: "Transcendental" named a mathematical class, while the property the effect promises is that the bits are the same on every machine; `cbrt` and `hypot` are algebraic and still differ between C libraries, and the `_approx` operations differ between processors. The property is stated and the two lists follow from it.

- READER EDIT rule=STD-27
  - old: the transcendental ones are
  `[DET-2]`'s platform functions.
  - new: those of them whose result IEEE 754 does not fix are `[DET-2]`'s platform functions, and carry `Nondet`.
  - why: The cross-reference repeated the word that left `cbrt` and `hypot` undecided; it now claims what DET-2 says.

**remaining_doubt**: E4070's help for `cbrt` and `hypot` has no exact line to write, since DET-4 gives `std.math.det` no `cbrt` or `hypot` (`det.pow(x, 1.0 / 3.0)` for a positive `x`, or `det.sqrt(x * x + y * y)` with the overflow `hypot` avoids); whether `std.math.det` should grow them is the lead's. The `std.math` vector and rotation methods carry `Nondet` through `[EFF-1]` exactly where `std/src/math/mod.em` calls a platform function (`from_angle`, `from_axis_angle`, `slerp`, `perspective_rh` call `angle.sin()`, line 1202), and `rsqrt` is `1.0 / x.to_real().sqrt()` (line 238), so the edit's two lists match the file; no owner verdict touches this.

**REFUTER reason**: Confirmed: `[DET-2]` "`Nondet` is introduced by exactly: … a transcendental function from the platform library instead of `std.math.det`" admits `cbrt` and `hypot` (algebraic), while the §X effect table defines `Nondet` as "a result that may differ between runs or machines", and the owner's own note (`docs/AUTOPILOT.md` §4: "`cbrt(27.0)` is `3.0` with MSVC and `3.0000000000000004` with glibc (`[DET-2]`)") shows `cbrt` does; `[STD-27]` "A method is the C library's function of its name … the transcendental ones are `[DET-2]`'s platform functions" repeats the word. `[SIMD-4]`'s "approximations named `_approx` (`rsqrt_approx`, `rcp_approx`)" are in no entry at all. A `@deterministic fn` using them compiles and `[DET-1]`'s cross-machine promise is broken; `[DET-4]` "`sqrt` is IEEE 754's, correctly rounded" and `[STD-3]` "a fused multiply-add with a single rounding, on every target" decide the exclusions the reader lists. No verdict touches it (ODR-136 is an agent's ruling). Narrowed for one omission at the same cause: `[TYP-30]` "A float base with a float or integer exponent is `pow`", so `x ** 2` on a float is the platform's `pow` and `Nondet` by the new entry, which the entry should say so that `E4070` on an operator names the operator and a line to write; the edit below is the reader's with that clause and the help added.

- BETTER EDIT rule=DET-2
  - old: a
  transcendental function from the platform library instead of `std.math.det`;
  - new: a floating-point function the platform's C library computes whose result IEEE 754 does not fix to one correctly rounded answer (`[STD-27]`'s methods `sin` through `hypot`, `cbrt`, `hypot` and `pow` among them, and `a ** b` on a float base, which is `pow` (`[TYP-30]`); not `sqrt`, `fma`/`mul_add`, `abs`, `copysign`, `floor`, `ceil`, `trunc`, `round`, `round_half_away` or `fract`, which every target computes alike, nor `rsqrt`, `lerp` and `smoothstep`, which `std.math` builds from those), where `std.math.det`'s is not used, `E4070` naming the function or operator and the line to write ("`boost` is `@deterministic`, and `x ** 2` (line 5) is the platform's `pow`, whose last bits differ between C libraries; write `x * x`, or `det.pow(x, 2.0)`"); a SIMD `_approx` operation (`[SIMD-4]`), whose bits are the processor's own;
  - why: "Transcendental" named a mathematical class, while the property the effect promises is that the bits are the same on every machine; `cbrt` and `hypot` are algebraic and still differ between C libraries, `**` on a float is `pow` by `[TYP-30]`, and the `_approx` operations differ between processors. The property is stated and the lists follow from it.

- BETTER EDIT rule=STD-27
  - old: the transcendental ones are
  `[DET-2]`'s platform functions.
  - new: those of them whose result IEEE 754 does not fix are `[DET-2]`'s platform functions, and carry `Nondet`.
  - why: The cross-reference repeated the word that left `cbrt` and `hypot` undecided; it now claims what `[DET-2]` says.

## K56-unsafe-at-compile-time  reader=not_answered  refuter=confirmed  should_be=not_answered  safety=none

**answered_by**: CT-1: "Any function may be called there if its effect set contains nothing beyond `Alloc`, `Panic(Explicit)` and `RuntimeCheck` — no `Io`, `FFI`, `Sync`, `Block`, `Unsafe` or `Nondet`. Allocation is served by the evaluator's own heap."; §X effect table: "| `Unsafe` | the body contains `unsafe` or the function is `unsafe fn` |"; CT-2: "An operation that cannot run at compile time — an `extern` call, a thread, a clock, `unsafe` pointer access outside the evaluator's heap, I/O — is `E6010`, naming the call chain. A raw pointer made from an integer (`0 as *mut T`, `-1 as *mut T`) is a compile-time value: it is an address, read by nothing, and a `const` of pointer type may hold one; a pointer into the evaluator's heap may not leave it (`E6010`) (ODR-145)."; CT-7: "A `comptime:` block at item level, or as a statement, runs once during compilation for its checks."; CT-4: "The evaluator computes integer and floating-point results exactly as the target does, including overflow panics and the target's pointer size, endianness and layout."; UNS-6: "Inline assembly is `unsafe asm("…", …)` on toolchains that support it and `E5090` otherwise."

**program**: fn crc_of(i: u32) -> u32:
    c = i
    for _ in 0..8:
        c = (c >> 1) ^ (0xEDB88320 & (0 - (c & 1)))
    return c

fn crc_table() -> [u32; 256]:
    t: [u32; 256] = [0; 256]
    unsafe:
        p = t.as_mut_ptr()                 # a pointer into the evaluator's heap, never leaving it
        for i in 0..256:
            p.add(i).write(crc_of(i as u32))
    return t

const CRC: [u32; 256] = comptime(crc_table())

comptime:                                  # the same writes, written in the block itself
    u: [u32; 4] = [0; 4]
    unsafe:
        u.as_mut_ptr().write(7)
    assert(u[0] == 7)

CT-1 refuses the `const` line (`crc_table`'s effect set holds `Unsafe`, which the table gives any body containing `unsafe`, and `[EFF-1]` passes up), so a library's private `unsafe:` block (`AnyError.downcast_ref`, `std/src/core.em`) bars every caller at compile time. CT-2 refuses only `unsafe` pointer access *outside* the evaluator's heap and lets a pointer into it exist, and CT-1's effect test is a test of called functions, so the `comptime:` block's own `unsafe:` write runs. The two rules decide the same operation two ways depending on whether it is written in a called function or in the block; an implementer cannot tell which rule is the intended one.

- READER EDIT rule=CT-1
  - old: — no `Io`, `FFI`, `Sync`,
  `Block`, `Unsafe` or `Nondet`.
  - new: — no `Io`, `FFI`, `Sync`, `Block` or `Nondet`; `unsafe` code runs there, each of its pointer operations checked by the evaluator as `[CT-2]` says, since the evaluator knows every value it holds.
  - why: The list barred the effect while CT-2 and ODR-145 describe checked pointer access inside the evaluator's heap; removing the effect from the list makes the two rules one, and a library's private `unsafe:` block no longer bars its callers silently. The better shape, not the cheaper one: the evaluator sees every access, so it refuses the bad one instead of all of them.

- READER EDIT rule=CT-2
  - old: — an `extern` call, a thread, a clock, `unsafe` pointer access outside the
  evaluator's heap, I/O — is `E6010`, naming the call chain.
  - new: — an `extern` call, a thread, a clock, inline assembly (`[UNS-6]`), I/O, and, in `unsafe` code, a read or write through a pointer that does not point at the bytes of a value the evaluator holds and has not ended (a pointer made from an integer, one past a value's end, one into a value already ended, a foreign address), the evaluator keeping the extent and the life of every value it makes — is `E6010`, naming the operation and the call chain ("`comptime(crc_table())` (line 15) → `crc_table` (line 7) → `p.add(i).write(…)` (line 12) writes past the end of `t` when `i` is 256; the table has 256 entries: write `for i in 0..256:`").
  - why: "Outside the evaluator's heap" named the property loosely; the check the evaluator can make is stated (a live value's bytes), inline assembly is added to the list of what cannot run, and the message names the chain and the line, as E6010's other messages do.

**remaining_doubt**: The cheaper alternative (the cluster's option B) is as safe: keep `Unsafe` in CT-1's list and replace CT-2's clause with "`unsafe` code (`[EFF-1]`: a function whose body or callee holds an `unsafe` block)", so `comptime:` blocks and called functions are refused alike, with E6010's help giving the safe line (`t[i] = crc_of(i as u32)`); it costs every compile-time user of a library whose implementation holds an `unsafe:` block. No verdict decides it. Under A, `UnsafeCell`, `transmute`-like reinterpretation and `copy_nonoverlapping` run by CT-4's target layout, and observing a pointer as an integer stays `Nondet` and so barred.

**REFUTER reason**: Confirmed: `[CT-1]` "Any function may be called there if its effect set contains nothing beyond `Alloc`, `Panic(Explicit)` and `RuntimeCheck` — no `Io`, `FFI`, `Sync`, `Block`, `Unsafe` or `Nondet`" with the §X table's "`Unsafe` | the body contains `unsafe` or the function is `unsafe fn`" bars `crc_table()` from `comptime(crc_table())`, while `[CT-2]` "`unsafe` pointer access outside the evaluator's heap … is `E6010`" and "a pointer into the evaluator's heap may not leave it (`E6010`) (ODR-145)" presuppose that pointer access inside the evaluator's heap runs, and `[CT-1]`'s test is of called functions' effect sets, so the same write runs in a `comptime:` block (`[CT-7]`) and is refused in a function it calls. Two rules, one operation, two answers: an implementer cannot choose. The reader's option A makes them one in the direction `[CT-2]` and ODR-145 already describe (a checked evaluator that knows the extent and life of every value it holds, as `[CT-1]`'s "Allocation is served by the evaluator's own heap" implies), costs nothing while the program runs, keeps `Nondet` (pointer-as-integer) and `FFI` barred, adds inline assembly (`[UNS-6]`) to what cannot run, and gives `E6010` a chain and a line as its other messages have. It departs from no verdict. The cheaper option B (bar every `unsafe` block, in the comptime block too) is as safe but makes a library's private `unsafe:` block (`AnyError.downcast_ref`) bar every compile-time caller, the shape the owner's condition on the delegation rejects ("the better shape, never the cheaper one"). An `unsafe fn` whose `@safety` obligation is not about memory (`from_utf8_unchecked`) can still materialise a broken invariant, but that program contains `unsafe` and is outside `[PHIL-10]` at run time as well; no promise is newly broken.

## K69-soa-flattened-field  reader=not_answered  refuter=narrowed  should_be=not_answered  safety=none

**answered_by**: SOA-1: "A flattened field has no column of its own: `ps.pos` is `E2072` ("`pos` is `@soa(flatten)`, so `ps` has no `pos` column; name one of its columns: `ps.pos.x`, `ps.pos.y`, `ps.pos.z`"), while `ps[i].pos` is a `Vec3` gathered from them by `copy of`, and `ps[i].pos.x` is a place in the column `pos.x`."; SOA-6: "A proxy's field `ps[i].f` is a place in column `f` at index `i`; reading or writing it touches that column only, and no `T` is assembled."; SOA-6: "`copy of ps[i]` gathers a `T` of your own (for `T: needs Clone`) by the proxy's `load()`, the method `copy of` runs here as it runs `clone` elsewhere [...] `ps[i] = v` scatters a value."; SOA-6: "A `T` with a `drop` is otherwise an `SoA` element like any other: its field `ps[i].f` is a place of its column, as `b.f` is a place of `b`. The same holds for a flattened field's struct (`@soa(flatten)`)."; E2072 registry row: "a flattened field's name as a column (`ps.pos`) | `[STD-13]`, `[SOA-1]`"

**program**: Implementer's question (the read side is now decided: `ps.pos` is `E2072` with its message, `ps[i].pos` a gathered `Vec3`, `ps[i].pos.x` a place):

from std.math import Vec3

struct Particle:
    @soa(flatten)
    pos: Vec3
    vel: Vec3

fn main():
    ps = SoA[Particle].with_capacity(16)
    ps.push(owns Particle(pos=Vec3.ONE, vel=Vec3.ZERO))
    ps[0].pos = Vec3.ZERO          # SOA-6's "a place in column f" needs a column `pos`, which does not exist;
                                   # SOA-1 says `ps[i].pos` is a gathered value, so this is either an
                                   # assignment to a non-place (which code?) or a scatter into pos.x, pos.y,
                                   # pos.z, as `ps[i] = v` scatters an element; nothing says which
    ps[0].pos += Vec3.ONE          # the same for a compound assignment

- READER EDIT rule=SOA-1
  - old: while `ps[i].pos` is a `Vec3` gathered from them by `copy
  of`, and `ps[i].pos.x` is a place in the column `pos.x`.
  - new: while `ps[i].pos` is the element's `pos` over those columns, as `ps[i]` is the element over all of them (`[SOA-6]`): read, it is a `Vec3` gathered from them by `copy of`; `ps[i].pos = v` scatters `v`'s fields into them, as `ps[i] = v` scatters an element, through a name that may change `ps` (`E3023` otherwise), and `ps[i].pos += d` is the gather, the operation and the scatter; `ps[i].pos.x` is a place in the column `pos.x`.
  - why: The new sentence decided the read and not the write; stating that a flattened field behaves as the element does (gather on read, scatter on write) adds no type and no special case, and the compound form follows.

**remaining_doubt**: A scatter into a flattened field is one store per column, the same stores as writing `ps[i].pos.x`, `.y`, `.z`, so no §X.4 row is added (the existing `ps[i] = v` scatter has none; the gather row is for elements with `drop`). Whether `ps[i].pos` read without `copy of` yields the `Vec3` (a `Copy` struct) or needs `copy of` as `ps[i]` does is left as SOA-1 now has it ("gathered from them by `copy of`"); the lead may want the words "`copy of ps[i].pos`" if the latter is meant.

**REFUTER reason**: Confirmed as undecided: `[SOA-6]` "A proxy's field `ps[i].f` is a place in column `f` at index `i`" needs a column `pos`, which `[SOA-1]` denies ("A flattened field has no column of its own"), and `[SOA-1]`'s new sentence decides only the read ("`ps[i].pos` is a `Vec3` gathered from them by `copy of`"); nothing says what `ps[0].pos = Vec3.ZERO` or `ps[0].pos += d` is. Narrowed to a better fix: the reader's edit makes `ps[i].pos` a value on read and a scatter on write, which gives a flattened field a shape the whole element does not have (`[SOA-6]`: "`ps[i]` is an **element proxy**: `SoARef[T]`, a read-only name of element `i`" and "`copy of ps[i]` gathers a `T` of your own … `load()` written at a use is `E1010` (\"write `copy of ps[0]`\")"), so `ps[i]` would need `copy of` while `ps[i].pos` would not, and compound assignment would exist for the field and not for the element. The property is that a flattened field is a proxy over its own columns, with everything `[SOA-6]` gives a proxy; a compound assignment is then defined once, for every proxy, as the gather, the operation and the scatter, the stores that writing its fields one by one would make, so no §X.4 row (the existing gather row is for elements with `drop`). No verdict touches `SoA`.

- BETTER EDIT rule=SOA-1
  - old: while `ps[i].pos` is a `Vec3` gathered from them by `copy
  of`, and `ps[i].pos.x` is a place in the column `pos.x`.
  - new: while `ps[i].pos` names the element's `pos` over those columns as `ps[i]` names the element over all of them (`[SOA-6]`): a proxy, read-only through `ps[i]` and changeable through `ref mut ps[i]` or a name that may change `ps`, whose field `ps[i].pos.x` is a place in the column `pos.x`; `copy of ps[i].pos` gathers a `Vec3` of your own from them (`load()` at a use is `E1010`, as for `ps[i]`); `ps[i].pos = v` scatters `v`'s fields into them, as `ps[i] = v` scatters an element, and `ps[i].pos += d` is a proxy's compound assignment (`[SOA-6]`).
  - why: The sentence decided the read and not the write, and gave the flattened field a shape the element does not have; stating that a flattened field is a proxy over its columns adds no type and no special case, and the write and compound forms follow from `[SOA-6]`.

- BETTER EDIT rule=SOA-6
  - old: `ps[i] = v` scatters a value.
  - new: `ps[i] = v` scatters a value, through a name that may change `ps` (`E3023` otherwise); `ps[i] op= v`, for a `T` with the operator, is the gather, the operation on the gathered value and the scatter, one load and one store per column, the stores that writing its fields one by one would make (the same for a flattened field's proxy, `ps[i].pos += d`, `[SOA-1]`).
  - why: Compound assignment on a proxy was unsaid for the element and for a flattened field alike; defined once where the scatter is, it costs the same stores as the per-field writes and needs no row of §X.4.

## K72-comptime-read-file  reader=not_answered  refuter=confirmed  should_be=not_answered  safety=none

**answered_by**: CT-2: "Two inputs are provided: `comptime.read_file(path)` (relative to the root of the package whose source holds the `comptime.read_file` call, the folder of that package's `ember.toml`, whatever package's item is being evaluated and whatever package is built; the path is resolved with every symbolic link followed and must then lie under that root, `E6010` otherwise [...]) and `comptime.env(name)`, which a call written in the root package's source may make for any variable [...] Both are recorded as build inputs."; CT-1: "Any function may be called there if its effect set contains nothing beyond `Alloc`, `Panic(Explicit)` and `RuntimeCheck` — no `Io`, `FFI`, `Sync`, `Block`, `Unsafe` or `Nondet`."; CT-4: "A result is cached with its inputs (source, `read_file` contents, `env` values)."; CT-5: "Its type is the read-only view of the value: `Span[T]` for an `Array[T]`, `StrView` for a `String`, `ref T` for any other value that owns heap memory, with the `static` region (`[LT-3]`)"; CT-6: "`comptime(e)` is an expression whose value is `e` evaluated at compile time. `e` may refer to constants, generic parameters, and statics initialised at compile time, not to run-time locals (`E6005`)."; STD-22: "Every operation that touches the outside world returns `Result[T, io.Error]` and carries `Io`"

**program**: Implementer's questions, not decidable from the text:

fn sky_shader() -> String:                      # or StrView? CT-2 gives `comptime.read_file` no result type
    return comptime.read_file("shaders/sky.wgsl")   # Io (STD-22: every read of the world), which CT-1 bars
                                                    # at compile time, or not an effect at all?

const SKY_LEN: int = comptime(sky_shader()).len()   # accepted only if read_file carries no Io

fn main():
    src = sky_shader()      # at run time: the text read when compiling, a read now, or E6010?
    println(src.len())

Also unsaid: a missing file, a file that is not UTF-8, and `comptime.env`'s result for an unset variable.

- READER EDIT rule=CT-2
  - old: Both are recorded as build
  inputs.
  - new: `comptime.read_file(path) -> StrView` is the file's text (`E6010` when the file is missing, unreadable or not UTF-8, naming the path and the call chain), and `comptime.env(name) -> Option[StrView]` the variable's value, `None` when it is unset; each result is kept in the image as constant data with the `static` region (`[CT-5]`, `[LT-3]`). Both run only while compiling, wherever they are written: the call is evaluated as a `comptime(e)` is, so its argument is a compile-time value (`E6005` otherwise: "`comptime.read_file(name)` (line 2) reads a file while compiling, and `name` is known only while running; give it the path as text: `comptime.read_file(\"shaders/sky.wgsl\")`"), the read is the compiler's and not the program's, so neither carries `Io`, and a function that calls one runs at run time with what was read when it was compiled (`sky_shader()` gives the same view on every path, and no file is opened while the program runs), as C's `#embed`. Both are recorded as build inputs.
  - why: The two inputs had a root and a security rule but no type, no effect and no meaning outside compile-time evaluation; stating them as constants read when compiling, wherever written, costs nothing while running, keeps CT-1's bar on `Io`, and answers the missing-file and unset-variable cases.

**remaining_doubt**: The cluster's option B (`E6010` where such a function is called outside what compile time evaluates, with `comptime(sky_shader())` as the help) is as safe and simpler for the compiler; A is chosen for the program's intent (embed the file) and the zero run-time cost, with no verdict either way. A binary embed (`Span[u8]`, as `fs.read` pairs with `fs.read_to_string`) has no form; the lead may add `comptime.read_bytes(path) -> Span[u8]` beside `read_file` if game assets are meant to be embedded.

**REFUTER reason**: Confirmed: `[CT-2]` gives `comptime.read_file(path)` a root ("relative to the root of the package whose source holds the `comptime.read_file` call") and a security rule ("must then lie under that root, `E6010` otherwise") and `comptime.env(name)` a permission rule, but no result type, no effect and no meaning outside compile-time evaluation; `comptime` has no row in §XV.1 (the rows are `std.core`, `std.math`, `std.ser`, …), so the Part XV preamble's "`std/**/*.em` holds the exact signatures" does not reach it, and `std/src` has no `read_file`. `[CT-1]` bars `Io` ("no `Io`, `FFI`, `Sync`, `Block`, `Unsafe` or `Nondet`") and `[STD-22]` makes "Every operation that touches the outside world … carries `Io`", so an implementer who gives the read `Io` refuses it at compile time, its only purpose; a missing file, a non-UTF-8 file and an unset variable are unsaid. The reader's edit decides all of it at the cause, at zero run-time cost (the read is the compiler's, the result constant data with the `static` region by `[CT-5]`, `[LT-3]`), as Rust's `include_str!` and C's `#embed` do; `E6005` for a run-time argument matches its registry row ("`comptime(e)` refers to a run-time local", `[CT-6]`); `[CT-4]`'s cache ("A result is cached with its inputs (source, `read_file` contents, `env` values)") and "Both are recorded as build inputs" are kept. Option B (`E6010` outside compile-time evaluation, as §XIV.2 does for `reflect[T]()`) is as safe; A is the better shape for the program's intent and no verdict decides between them.

## K73-det-ln-name  reader=answered_differently  refuter=confirmed  should_be=answered_differently  safety=none

**answered_by**: DET-4: "`std.math.det` provides `sin`, `cos`, `tan`, `exp`, `log`, `pow`, `atan2` and `sqrt` for `f32` and `f64` returning the same bits on every supported target."; STD-21: "`std.math` provides `PI`, `TAU`, `E`; `sin`, `cos`, `tan`, `asin`, `acos`, `atan`, `atan2`, `sinh`, `cosh`, `tanh`, `exp`, `exp2`, `ln`, `log2`, `log10`, `pow`, `sqrt`"; STD-27: "A method is the C library's function of its name for `f64` and its `f` form for `f32` (`ln` is C's `log`, `round` its `nearbyint`, `round_half_away` its `round`, `mul_add` its `fma`)"; STD-13: "**One naming convention.** Names are `snake_case` words joined by underscores [...] Where Python's name for a **method** differs (`append`, `strip`, `startswith`, `upper`, `splitlines`), there is no second name"; Part XV preamble: "For a module whose row names another Part, that Part alone is the authority: where it is silent the construct is `E0901`, and a file of `std` that disagrees with it is a defect of the file."

**program**: 

- READER EDIT rule=DET-4
  - old: `std.math.det` provides `sin`, `cos`, `tan`, `exp`, `log`, `pow`, `atan2` and `sqrt` for
  - new: `std.math.det` provides `sin`, `cos`, `tan`, `exp`, `ln`, `pow`, `atan2` and `sqrt`, each named as `[STD-21]`'s function of the same meaning (`det.log` is `E1010`, whose help is "the natural logarithm is `ln` in `std.math` and `std.math.det` alike: write `det.ln(p)`"), for
  - why: One function had two names in two modules, so code moved into a `@deterministic` function had to rename its call; `std.math` already explains why `ln` is not C's `log` (`[STD-27]`), and `det`'s functions are then a subset of `std.math`'s by name.

**remaining_doubt**: The current text decides it (`det.log`, safe, a naming matter only), against the checker's recommended `det.ln`; the edit is the consistency fix and the lead may keep `log`. If applied, `std/src/math/det.em:109` (`pub fn log`) becomes a defect of the file by Part XV's preamble, and CT-4's example (`det.sin`) is unaffected. E1010 is the registry's code for a name "written with a word instead"; if the lead prefers E2072's "no method or field of that name" family for a module function, that row's wording would need "or function".

**REFUTER reason**: The text decides it (`[DET-4]` "`std.math.det` provides `sin`, `cos`, `tan`, `exp`, `log`, `pow`, `atan2` and `sqrt`"), safely, a naming matter only; but it contradicts the document's own reason for `std.math`'s name: `[STD-21]` lists "`exp`, `exp2`, `ln`, `log2`, `log10`" and `[STD-27]` says "(`ln` is C's `log`, …)", so the natural logarithm is `ln` in one module and C's `log` in its deterministic twin, and code moved into a `@deterministic` function renames its call. The brief's consistency goal and `[STD-13]`'s "there is no second name" point at one name; `E1010`'s registry row ("written with a word instead") fits the refusal; `det.sin` is the only `det` name used elsewhere in the text (`[CT-4]`, §XIV example), so nothing else changes. The edit adds no cost and departs from no verdict; `std/src/math/det.em:109` (`pub fn log`) then becomes a defect of the file by the Part XV preamble, which the lead must know.

## K74-anyerror-core-layer  reader=answered  refuter=confirmed  should_be=answered  safety=none

**answered_by**: STD-6: "Each module's row in §XV.1 names its layer, and a name whose layer differs from its module's is marked there (`AnyError`, `std.core`'s name but the **alloc** layer's type, `[ERR-8]`). A package that declares `[package] layers = ["core"]` may use only core; this is how firmware and kernels use Ember. A use of a name of a layer the package does not declare is `E1042`, naming the layer and the line to write ("`Result[int]` means `Result[int, AnyError]`, and `AnyError` needs the `alloc` layer, which `sensor` does not declare; name the error type: `Result[int, SensorError]`")."; §XV.1 module table, `std.core` row: "layer: **core** (`AnyError` is **alloc**, `[STD-6]`)"; ERR-9: "The error parameter defaults to `AnyError` (§XIII.2), so `fn load(path: StrView) -> Result[Texture]` is a function that can fail with any error."; ERR-11: "Converting an error into `AnyError` allocates, once, on the failure path; that path carries `Alloc` (`[EFF-1]`)."; BLD-11: "A package that omits a `[STD-6]` layer cannot use it or depend on a package that does."; E1042 registry row: "a use of a name of a layer of `std` the package does not declare | `[STD-6]`"

**program**: 

**remaining_doubt**: None of substance: the finder's program (`-> Result[int]` in a `layers = ["core"]` package) is `E1042` at the signature with the exact message STD-6 gives, the cluster's recommended option A; `Err(AnyError.msg(String("short frame")))` is `E1042` for `AnyError` and for `String` alike. Verdicts 164 to 166 do not touch it.

**REFUTER reason**: Decided, safely, by the quoted text: `[STD-6]` "a name whose layer differs from its module's is marked there (`AnyError`, `std.core`'s name but the **alloc** layer's type, `[ERR-8]`) … A use of a name of a layer the package does not declare is `E1042`, naming the layer and the line to write (\"`Result[int]` means `Result[int, AnyError]`, and `AnyError` needs the `alloc` layer, which `sensor` does not declare; name the error type: `Result[int, SensorError]`\")"; the §XV.1 row reads "layer: **core** (`AnyError` is **alloc**, `[STD-6]`)"; `[ERR-9]` gives the default ("The error parameter defaults to `AnyError`"), `[BLD-11]` closes the dependency route, and the `E1042` registry row names `[STD-6]`. `String` in `Err(AnyError.msg(String("short frame")))` is **alloc** by `[STD-6]`'s layer list and is refused the same way. No verdict touches it; nothing to add.

# MISSED

## MISSED: `dyn I` has no `ThreadShared` rule, so a `static` or `ref` of `owns dyn I` across threads is undecided

**rules**: `[THR-8]`: "`dyn I` when `I` requires it (`interface I: needs Send:`)" — stated for `Send` only. `[THR-9]`: "Structural types are `ThreadShared` when every component is. `ref mut T`, `MutSpan[T]`, `sharedref mut T`, `Cell`, `UnsafeCell`, raw pointers, …, `shares T`, `shares mut T`, and a `weakref T` link are not" — `dyn I` appears in neither list, and the §IV.8 `ThreadShared` row ("every component is `ThreadShared`") has no components to read for a kind known only while running. `[STA-1]`: "`T` MUST be `ThreadShared` (`E7002`)". `[THR-8]`: "`ref T` and `Span[T]` when `T` is `ThreadShared`".

**program**:
```
interface Shape:
    fn area() -> float

class Square:
    side: float = 1.0
    fn area() -> float: return this.side * this.side

static UNIT: owns dyn Shape = owns Square()      # E7002 or accepted? THR-9 says nothing of `dyn I`

fn main():
    s: owns dyn Shape = owns Square()
    h = thread.spawn(owns fn() => println(UNIT.area()))   # a `ref dyn Shape` is Send only if `dyn Shape` is ThreadShared
    h.join()

An implementer who reads `dyn I` as `ThreadShared` lets a `dyn Shape` whose kind holds a `Cell` be read from two threads at once; one who reads it as never `ThreadShared` refuses every `static` of an interface type and every `ref dyn I` given to a thread with no line to write. The reader's K20 edit (`AnyError` is not `ThreadShared`) rests on the second reading without a rule saying it.
```

- MISSED EDIT rule=THR-9
  - old: a `weakref
  thread_shared T` for such a `T`, and a channel's `Sender[T]` for a `Send` `T` are (`[THR-15]`).
  - new: a `weakref
  thread_shared T` for such a `T`, and a channel's `Sender[T]` for a `Send` `T` are (`[THR-15]`); `dyn I` is when `I` requires it (`interface I: needs ThreadShared:`), as it is `Send` when `I` requires that (`[THR-8]`), since the kinds behind a `dyn I` are known only while the program runs; so `AnyError`, which holds a `dyn Error`, is neither (`[ERR-8]`).
  - why: `[THR-8]` decides `Send` for `dyn I` by the interface's requirement and `[THR-9]` is silent on the same type; the same mechanism decides `ThreadShared`, and the `AnyError` fact the K20 edit states then follows from a rule.

## MISSED: `thread_shared T using A` is allowed by `[OBJ-1]`, `[RC-4]` and `[TYP-46a]` and `E2114` by `[OBJ-4]` and `[ALC-1]`

**rules**: `[OBJ-1]`: "A `thread_shared` value made `using` an instance (`[ALC-1]`) lies in memory the runtime did not lay out, so it is marked handed over at the line that makes it (`[RC-4]` point 4) and no thread asks which thread made it." `[RC-4]` point 4: "A `thread_shared` value made `using` an instance of a manager that may have several instances (`[ALC-1]`) is marked handed over at the line that makes it". `[TYP-46a]`: "`thread_shared T using A` with an `A` that is not `ThreadShared` is `E7015` at the making" (so with a `ThreadShared` `A` it is made). Against them `[OBJ-4]`: "A `thread_shared` value's block always comes from the runtime allocator (`[RT-1]`), whatever manager its class names: `thread_shared` before a value made `using m` is `E2114`", and `[ALC-1]`: "`using A` after a type whose value has heap memory of its own (…; never a `thread_shared` value, whose block carries its making thread's record, `[OBJ-4]`, `E2114`)". The reader quoted `[OBJ-1]`'s sentence for K13 and did not see that two other rules refuse the line it describes.

**program**:
```
from std.arena import Arena
import std.thread

class Names:
    items: Array[String] = []

fn main():
    level = Arena()
    names = thread_shared Names() using level     # OBJ-1, RC-4 point 4 and TYP-46a describe this value;
                                                 # OBJ-4 and ALC-1 make the line E2114
    h = thread.spawn(owns fn() thread_shared names: names.items.len())
    println(h.join())

An implementer cannot decide whether the line compiles; one who follows `[OBJ-1]` must also decide whether the arena may be reset while the other thread holds an owner (`[LT-4]`, `E3063`, which `[RC-4]` point 4 says the handed-over mark exists to make safe).
```

- MISSED EDIT rule=OBJ-4
  - old: A `thread_shared` value's block always comes from the runtime allocator
  (`[RT-1]`), whatever manager its class names: `thread_shared` before a value made `using m` is
  `E2114` ("a `thread_shared` value's block carries its making thread's record, which only the
  runtime allocator keeps; remove `using m`: `names = thread_shared Names()`").
  - new: A `thread_shared` value's block comes from the runtime allocator
  (`[RT-1]`), whose chunk header records its making thread (`[OBJ-1]`), unless the making line says `using m`: then it lies in the instance, with no chunk header to read, and is marked handed over at that line (`[RC-4]` point 4, `[TYP-46a]`), so no thread asks which thread made it and its end is never delayed past the instance's reset (`[ALC-5]`, `[LT-4]`).
  - why: Two rules refused what three others describe; the mechanism (`[RC-4]` point 4's handed-over mark at the making line, N84) exists for this case, so the refusal is the stale half. If the lead prefers the refusal, `[OBJ-1]`'s sentence, `[RC-4]` point 4's clause and `[TYP-46a]`'s `E7015` must go instead.

- MISSED EDIT rule=ALC-1
  - old: a class object, an `Array`, `Map`, `Set` or `String`; never a
  `thread_shared` value, whose block carries its making thread's record, `[OBJ-4]`, `E2114`) names
  - new: a class object, an `Array`, `Map`, `Set` or `String`; a
  `thread_shared` value too, which is then marked handed over where it is made, `[OBJ-4]`, `[RC-4]` point 4) names
  - why: The same contradiction, at the rule that introduces `using`.

## MISSED: `E4070`'s help has no line to write for a platform function `std.math.det` does not provide

**rules**: `[DET-4]`: "`std.math.det` provides `sin`, `cos`, `tan`, `exp`, `log`, `pow`, `atan2` and `sqrt`". `[STD-21]`: "`sin`, `cos`, `tan`, `asin`, `acos`, `atan`, `atan2`, `sinh`, `cosh`, `tanh`, `exp`, `exp2`, `ln`, `log2`, `log10`, `pow`, `sqrt`, `rsqrt`, `cbrt`, `hypot`". `[DET-2]`'s `E4070` message gives a line to write ("give each task its own result and join them in order: write …"), and the brief's rule 2: "the message gives the line to write". With the K55 edit, `cbrt`, `hypot`, `asin`, `acos`, `atan`, `sinh`, `cosh`, `tanh`, `exp2`, `log2` and `log10` carry `Nondet` and `det` has none of them, so the refusal of `math.cbrt(x)` in a `@deterministic` function can name no deterministic function; `det.pow(x, 1.0 / 3.0)` is wrong for a negative `x` and `det.sqrt(x * x + y * y)` overflows where `hypot` does not.

**program**:
```
import std.math
from std.math import det

@deterministic
fn fall_off(d: float) -> float:
    return math.cbrt(d)        # E4070 after the K55 edit; the message must give the line to write, and det has no cbrt

@deterministic
fn dist(x: float, y: float) -> float:
    return math.hypot(x, y)    # the same; det.sqrt(x * x + y * y) overflows for large x
```

- MISSED EDIT rule=DET-4
  - old: `std.math.det` provides `sin`, `cos`, `tan`, `exp`, `log`, `pow`, `atan2` and `sqrt` for
  - new: `std.math.det` provides, under the names `[STD-21]` gives them, every function of `std.math` whose platform form is `Nondet` (`[DET-2]`): `sin`, `cos`, `tan`, `asin`, `acos`, `atan`, `atan2`, `sinh`, `cosh`, `tanh`, `exp`, `exp2`, `ln`, `log2`, `log10`, `pow`, `cbrt`, `hypot` and `sqrt`, so that `E4070`'s help for a `std.math` call is always `det`'s function of the same name ("`fall_off` is `@deterministic`, and `math.cbrt(d)` (line 6) is the platform's `cbrt`, whose last bits differ between C libraries; write `det.cbrt(d)`"), for
  - why: The property (a deterministic twin for every platform function the effect refuses) gives every `E4070` on a `std.math` call one line to write, and stops the list drifting from `[STD-21]`'s; it folds in the K73 rename. The cost is `det`'s own implementations (ODR-038 already requires them to be its own), none while a program that does not call them runs.
