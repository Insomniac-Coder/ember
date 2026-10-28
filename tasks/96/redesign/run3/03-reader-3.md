# 03 reader-3: the words of spec lines 3177-5015 (Parts X-XVI)

Slice: `docs/spec-source/Ember_v0.9.10_Hardened_35.md` lines 3177-5015: Part X effects and contracts,
XI threads, XII data-oriented code, XIII errors, XIV compile-time code, XV standard library, XVI calling C.
Power: `PowerLineStatus` printed `Online` at the start.

How I worked: I read the whole slice. A small script, `target/redesign3/03-reader-3/scan.py`, lists every
line of the slice where each word of `inputs/must-cover.md` and every `@name` appears inside code, so the
line numbers below come from it and from my reading. Every claim about what today's Ember accepts carries
the real output of `target/redesign3/bin/ember.exe check`, pasted in section 6; the snippets are
`target/redesign3/03-reader-3/t1..t9*.em`.

Terms used below:
- **the owner's test**: for each name in a line, point at the word that says what it is (the owner's own
  thing, a copy, or handed over) and the word that says whether it may change it (brief).
- **plain English**: a programmer who has never read the spec reads the word and guesses what it does.
- **sits on the thing**: the word is written on the item it describes and names it.
- **the agreed table**: rows 3-9 of `inputs/JOB.md`, binding.

## 1. The findings that matter most (short)

1. **`owned fn` on a lambda means two things by type** (lines 3495, 3535, 3563, 4899). Today
   `thread.spawn(owned fn() => count + 1)` copies the `int` `count` (using `count` afterwards is accepted,
   t3), while `owned fn() => name.len()` hands over the `String` `name` (using it afterwards is refused,
   E3040, t8). This is exactly JOB's "not acceptable" example, now with output. `owned` as a parameter
   (`owned f: fn()`, `owned self`, `owned text: String`) and as the C contract word (callee takes it) always
   means "handed over" and stays. Verdict: remove `owned` as a prefix on a lambda; what a lambda copies is
   named (`with copied x`, agreed row 6).
2. **`mut` and `MutSpan` say "may change" twice, and the spec does not agree with itself** (3576 vs 3618,
   3663, 3704, 4207). The spec's parallel-loop example `fn integrate(pos: MutSpan[float], ...)` writes
   `pos[i] += ...` with no `mut`; today that is refused with E3023 (t1), and the same function with
   `mut pos: MutSpan[float]` is accepted (t2). A `MutSpan` parameter without `mut` reads as "a view I may
   write through" and cannot be written through. One of the two words is redundant: merge.
3. **`Sync` is one word with three meanings** in this slice: an effect ("does locking, atomics, channel or
   thread work", 3186, `@nosync` 3240), a type property ("several threads may read one value at once",
   3445), and the class marker `@sync` ("handles may cross threads, counts are atomic, fields frozen after
   `init`", 3451-3475). The spec has to forbid its own natural reading: "A diagnostic MUST NOT describe
   `@sync` or `Sync` as thread-safe" (3475), and "It does not make methods atomic" (3473). Verdict: keep the
   effect and `@nosync` (there "sync" does mean synchronisation work); rename the type property and the
   class marker.
4. **`@static_safe` says "all safety proved at compile time" but forbids only aliasing checks** (3245,
   3260): bounds, overflow and stale-handle checks are allowed under it. A programmer who writes it on a hot
   loop expecting no run-time checks still gets them. Rename to say what it forbids, in the `@no...` pattern
   the other contracts use.
5. **`@fp(contract)` means "allow fusing `a * b + c`"** (3332, 3380, 4492), while everywhere else in this
   slice "contract" means a promise the compiler checks (Part X) or the facts of a C pointer (Part XVI).
   `@fp(contract)` reads like "keep strict float rules", the opposite of what it does. Rename.
6. **`SoA.get(i)` silently copies, `Array.get(i)` gives a reference** (3672, 3686 vs 4285). Same method
   name, different meaning by type; the copy is not named, against agreed row 9 ("a copy is only made when
   named"). Today `p = ps.get(0)` stays usable after `ps[0].life -= 0.5` (t6), so `p` does not borrow `ps`.
7. **`Box` and `ForeignBox`** (3535, 4031, 4129, 4924-4926, 4957; 4725, 4826, 4883-4886): `owned` replaces
   `Box` (JOB). `ForeignBox[T]` carries the same rejected word for "a C object this value owns and destroys".
8. **`Shared[T]` vs the agreed `shared`** (row 9 names `shared` as a way out): two spellings for one
   meaning if both stay. `SyncShared[T]` inherits finding 3.
9. **Agreed row 5's message writes `String(...)`, which today's Ember refuses**: `return String(word)` is
   E2020 "`String()` takes no arguments" with no help line (t9). The spec's spellings are
   `String.from(s)` and `s.to_string()` (4235). The row is binding, so either `String(s)` becomes a
   constructor or the builder must flag that the message names a form the language lacks.
10. **"handle" names four different things** (class handle 3440/3467, `Handle[T]` 3812, `@ffi(handle)`
    4775, `JoinHandle` 3503). The written words (`Handle`, `@ffi(handle)`, `JoinHandle`) all mean "a token
    that refers to something elsewhere" and keep; the spec's own prose term "class handle" (a counted
    pointer to a class object) is the odd one, a prose term for reader-2 and the builder.
11. **Two error codes each mean two unrelated things in this slice**: `E5015` is a variadic argument C
    would promote (4738) and an owning value in an exported signature (4959; the table at 5774 says only
    the second); `E7010` is a parallel loop's unprovable write (3589) and a non-`Sync` static reachable from
    an `any`-thread export (4966; the table at 5816 says only the first). For reader-4.

## 2. Keywords (reserved, II.4)

Columns: meaning at each place (lines); same meaning everywhere and for every type; plain English; sits on
the thing; verdict and reason.

| word | meaning in this slice (lines) | same everywhere / every type | plain English | sits on thing | verdict |
|---|---|---|---|---|---|
| `owned` | (a) parameter that takes ownership: `owned f: fn() -> R` 3503, 3627; `owned self` 3908-3918, 4376; `owned T` inside a function type 3908-3918; `owned text: String` 3937. (b) prefix on a lambda: `owned fn() => produce(tx)` 3495, "an `owned fn` capture" 3535, 3563, "MUST be an `owned fn`" 4899: captures are moved, except `Copy` ones (an `int`), which are copied (t3 vs t8). (c) C contract word "callee takes it" 4826. (d) prose "owned callable value" 3372, 3443. | No: (b) copies an `int` and hands over a `String` | (a), (c) yes; (b) no, it does not say what is copied | (a), (c) yes; (b) sits on the lambda, not on the captured names | **keep** (a), (c); **remove** (b): JOB's own example of a keyword that does different things by type; captures are named instead (agreed row 6, `with copied x`) |
| `ref` | type `ref T` (read-only reference) 3439, 3449, 3515, 3903, 3927, 4049, 4285, 4309-4327, 4577, 4826-4827; `ref mut T` 3448, 3457, 3512, 3800, 3825, 3903, 4286, 4310-4315, 4451, 4830-4832; expression `ref value` 4913 | yes | yes (with `mut`: "a reference that may change it", owner-approved on row 8) | yes | **keep**; agreed rows 4 and 8 use it |
| `mut` | `mut self` receiver 3360, 3459, 3903, 4271, 4430, 4451; `mut` parameter = changes reach the caller 3457, 3618, 3663, 3704, 3799, 3826, 4110, 4832; `ref mut` 3448 ff.; raw pointer `*mut T` 4012, 4725, 4926; `static mut` 4731. 4832: "`borrowed one exclusive` is written `p: ref mut T` or `mut p: T`" (two spellings, one meaning). 3576: `pos: MutSpan[float]` with no `mut` is refused today (t1, E3023). | "may change" everywhere; but on a parameter it also means "the caller's own value", which on a `match` binding it does not (JOB row 8, reader-2's slice) | "may change": yes | yes | **keep** the word; **merge** the duplicate spellings: `mut p: T` and `p: ref mut T` (4832) are one thing, and `mut` + `MutSpan` (finding 2) says it twice; the builder picks one spelling each |
| `with` | scoped block whose value is dropped at the block's end: `with g = m.lock():` 3422, 3513; `with scope = thread.scope():` 3428, 3551, 3557; `with s = jobs.scope():` 3619; `with debug.zone("name"):` 4184. Also an option list: `import c "h" with (include_paths = ...)` 4682, 4689 | No: block vs option list (and agreed row 6 adds a third: the capture list `fn() with copied score:`) | yes in each place | yes | **keep**; note for 05/07: three uses, the agreed row-6 one is placeholder spelling, the block and option-list ones read naturally |
| `unsafe` | block whose obligations the programmer answers for 3269, 3354, 3362, 4011, 4615, 4711, 4850; `unsafe fn` (calling needs `unsafe`) 3192, 3727, 4736, 4883; `unsafe extern "C":` (the programmer asserts these signatures) 4592, 4631; `unsafe overlay` (an assertion about C code) 4615, 4774, 4817; `unsafe assume_disjoint` 3736; contract word "leave the call unsafe" 4838; the effect `Unsafe` 3192 | Two directions, as in Rust: `unsafe fn` puts the duty on the caller; `unsafe extern`/`unsafe overlay`/block discharge it here | yes ("this part is not checked") | yes | **keep** (area E) |
| `extern` | C-ABI foreign block 3193, 4592, 4631; function-pointer type `extern "C" fn` 4733, 4895, 4908-4916, 4946; opaque type `extern type S` 4728; `extern static` 4731; exported definition `pub extern "C" fn` 4941 | yes: "uses C's calling convention / lives in C" | yes for C programmers | yes | **keep** |
| `static` | program-lifetime item 3478, 4000, 4033-4040, 4060; the "static region" (lives for the whole program) 3505, 3628, 4302, 4899; `from(static)` 4669, 4829, 4877; `extern static`/`static mut` 4731 | yes: "lives as long as the program" | yes | yes | **keep** |
| `const` | compile-time constant item 3991, 4000, 4013, 4042, 4060, 4696, 4703, 4732; C's `const T*` (read-only pointee) 4618, 4664, 4725, 4820 | two senses, C's is C's own | yes | yes | **keep** |
| `comptime` | `comptime(e)` 3991, 3999, 4004, 4038; `comptime:` block 3993, 4007; `comptime.read_file`/`env` 4014-4016 | yes | yes | yes | **keep**; note: `comptime(e)` of a heap type is cloned at each run-time evaluation (4038-4040), a copy no word names (builder: annoyance list) |
| `dyn` | interface value with dynamic dispatch 3205, 3367, 3437, 3927, 3933, 4926 | yes | jargon, but the owner kept it (`no_share(dyn Pool)`, JOB) | yes | **keep** |
| `fn` | function, lambda `fn() =>`, function type `fn(A) -> R` (3206 `@noalloc fn(A) -> R`), `safe fn`, `unsafe fn`, `once fn` | yes | yes | yes | **keep** |
| `yield` | generator suspension 3201; no `@must_drop` across it 3535; banned in `@parallel` 3583 | yes | yes | yes | **keep** |
| `defer` | `finally` → `defer:` 3967; skipped by `process.exit` 4573 | yes | yes | yes | **keep** |
| `override` | method override inherits contracts 3273; overlay entry replacing an earlier overlay's 4762-4765, 4809 | yes: "replaces an earlier definition of this name" | yes | yes | **keep** |
| `type` | associated type `type Real: Float` 4510; opaque `extern type S` 4728; overlay `type VkBuffer` 4776-4778, 4797 | yes | yes | yes | **keep** |
| `enum`, `struct`, `class` | 3856, 4729, 4780, 4879; 3658, 4051, 4727, 4880; 3414, 3451-3460, overlay `class` 4800 | yes | yes | yes | **keep** |
| `interface`, `implements` | 3438 (`interface I: Send:`), 3922, 4332-4333, 4445, 4510, 4514 | yes | yes | yes | **keep** |
| `import`, `pub` | 3410, 3487, 3616, 4682-4686, 4757 (`pub import c ... as vk`) | yes | yes | yes | **keep** |
| `as` | cast 3988, 4012, 4730, 4738; downcast `as?`/`as!` 3190, 3363; naming: `import ... as vk` 4686, `rename X as Y` 4789, 4797 | convert vs name, both Python's | yes | yes | **keep** |
| `in`, `not` | `for x in` 3419, 3497; `x in c` / `x not in c` 4352-4361 | yes | yes | yes | **keep**; `Not[D]` (ECS) is a library type, below |
| `and`, `or` | operators 3705 | yes | yes | yes | **keep** |
| `match` | 3868, 3967 | yes | yes | yes | **keep** |
| `for`, `break`, `continue`, `return` | 3419; 3559, 3583-3584; 3966 | yes | yes | yes | **keep** |
| `void` | `Result[void, E]` 3526, 3875, 3963, 4350, 4882; `-> void` 4271 | yes | yes | yes | **keep**; note `Result[void]` returns `Ok(())` (3879): `void` the type, `()` its value |
| `self`, `Self` | receiver 3459, 3903, 3908-3918, 4333, 4736; `Self` 4110, 4446 | yes | yes | yes | **keep** |
| `true`, `false` | 4154, 4255, 4888 | yes | yes | yes | **keep** |
| `is` | only as a method name, `AnyError.is[E]() -> bool` 3935 | n/a | yes | yes | **keep** (not used as the keyword here) |

Not in this slice as code (reader-1/2/4 judge them): `elif`, `else`, `if`, `let`, `pass`, `super`,
`virtual`, `where`, `while` (the four last appear only in prose).

## 3. Contextual and reserved words ([LEX-15])

| word | meaning in this slice (lines) | same everywhere | plain English | sits on thing | verdict |
|---|---|---|---|---|---|
| `safe` | `safe fn` inside an `unsafe extern` block: the programmer asserts a call cannot break memory safety 4594, 4596, 4613, 4634-4668 | yes | yes, and the surrounding `unsafe extern` says it is an assertion | yes | **keep** |
| `once` | `once fn(...)`: called at most once 3908-3918; `callback(once)` 4836, 4899-4901; `Once` type runs an initialiser once 3448, 3516 | yes: "at most once" | yes | yes | **keep** |
| `from` | `from m import x` 3411, 3487; C result lifetime `from(self)`, `from(p)`, `from(static)` 4669, 4829, 4877; `From[E]` conversion 3887, 3921-3922, 3934; `@from` 3930; `String.from(s)` 4235 | three senses (source module, source of a lifetime, conversion source), all "comes from" | yes | yes | **keep** |
| `extend` | `extend T implements I` 3922, 4510, 4514; overlay `extend` blocks 4855; also the method names `Array.extend` 4281, `Vec.extend` 4538 | keyword vs method name | yes | yes | **keep** (contextual, so the methods are allowed) |
| `some` | `some I` return: concrete type known to the compiler 3373 | yes | fair | yes | **keep** |
| `c`, `cpp` | `import c "h"` 4682-4689; `overlay c "h"` 4774, 4796; `c"..."` literal 4220, 4732; `@layout(c)` 3979; `"cpp"` 4796 | yes | yes | yes | **keep** |
| `threads` | contract word `threads(any \| main \| creator)` 4608, 4837, 4921, 4930; `@export(threads=...)` 4964; `#! threads` directive 4965, 4981 | yes | yes | yes | **keep** |
| `overlay`, `rename`, `hide` | overlay file and its items 4774-4818, 4789-4790 | yes | yes | yes | **keep** |
| `union` | `@layout(c, union) struct` 4727; method names `Set.union` 4327, `Aabb.union` 4555 | yes | yes | yes | **keep** |
| `async`, `await` | reserved 3639-3643 | n/a | yes | n/a | **keep** reserved |
| `macro` | reserved for user derives 4105; `@ffi(macro_fn)` 4700-4702 | yes | yes | yes | **keep** |

Not in this slice as code: `abstract`, `open`, `gen`, `language` (only the prose "mini-language" at 4173).

## 4. Attributes (`@name`) in this slice

| word | meaning (lines) | same everywhere | plain English | sits on thing | verdict |
|---|---|---|---|---|---|
| `@sync` | class marker: handles may cross threads, counts atomic, fields frozen after `init`, no `mut self` 3413, 3451-3475, 4934, 4966; today's E7003 confirms the frozen fields (t5) | yes as a marker, but shares the word `Sync` with an effect and a type property (finding 3) | **no**: reads "synchronised class"; the spec must forbid that reading (3473, 3475) | yes | **rename** (placeholder `@cross_thread`): fails plain English; a programmer reads it as "methods are safe to call from several threads" and writes a read-then-write race |
| `@nosync` | contract forbidding the `Sync` effect 3240, 3277 | yes | yes ("no synchronisation work") | yes | **keep**, once the type property and class marker stop sharing the word |
| `@noalloc`, `@noblock`, `@noio`, `@nolock` | contract forbidding `Alloc` / `Block` / `Io` / `Lock` 3239-3243, 3265-3266, 3798, 4133, 4200 | yes | yes | yes | **keep** |
| `@nopanic` | `@nopanic(explicit)`: forbids written panics and allocation failure; bare `@nopanic` is E0104 3244, 3263, 3287-3297 (t7) | yes | yes, with its required argument | yes | **keep** |
| `@static_safe` | forbids only `RuntimeCheck(Aliasing)`; bounds, overflow and stale checks allowed 3245, 3260, 3267, 3275, 3277; accepted today (t4) | yes | **no**: says "all checked at compile time" | yes | **rename** (placeholder `@no_alias_checks`): says what it forbids, as `@noalloc` does; a programmer cares because the name promises no run-time checks and bounds checks remain |
| `@deterministic` | contract: no `Nondet` 3246, 3307-3333 | yes | yes | yes | **keep** |
| `@realtime` | the manifest's contract set, "not a timing guarantee" 3247, 3295-3298 | yes | partly: reads as a deadline promise, which the spec disclaims (3297) | yes | **rename** (placeholder `@realtime_safe`, the industry term for "no alloc, no lock, no block"): the word promises timing it does not give |
| `@assume_noalloc` | expression, inside `unsafe` only: take this call as allocation-free 3269 | yes | yes | yes (on the call) | **keep** |
| `@parallel` | loop runs in chunks on the job system; `chunk=`, `reduce=[total: +]` 3577-3612, 3755 | yes | yes | yes | **keep** |
| `@simd` | request and report of vectorisation; `@simd(assert)` makes failure E4020 3729-3743 | yes | yes | yes | **keep** |
| `@fastmath` | float relaxations, `Nondet` 3316, 3332, 3755, 4456 | yes | yes | yes | **keep** |
| `@fp` | `@fp(contract)`: allow fused multiply-add 3332, 3380, 4492; accepted today (t4) | "contract" means something else everywhere else in the slice | **no** for `contract` | yes | **rename** the argument (placeholder `@fp(fuse)`): `contract` here is the C-standard jargon "FP contraction"; next to Part X's contracts it reads as "strict" |
| `@overflow` | `@overflow(wrap)` 3355, 3719 | yes | yes | yes | **keep** |
| `@must_drop` | value whose drop a safety guarantee needs; may not be forgotten, stored, returned 3533-3539, 3562 | yes | yes ("must be dropped") | yes | **keep** |
| `@must_use` | discarding a `Result` is W2190 3919 | yes | yes | yes | **keep** |
| `@derive` | generated implementations: `Copy`, `Clone`, `Eq`, `Debug`, `Display`, `Ord`, `Hash`, `Default`, `Error`, `Serialize`, `Deserialize`, `Zeroable`, `Component` 3790, 3855, 4085-4105 | yes | yes | yes | **keep** |
| `@error` | variant's display text 3857, 3859, 3928 | yes | yes | yes | **keep** |
| `@from` | variant converts from its field's error type 3930 | yes | yes | yes | **keep** |
| `@default` | enum's default variant 4097 | yes | yes | yes | **keep** |
| `@reflect` | run-time type information 4048-4068 | yes | yes | yes | **keep** |
| `@attribute` | marks a struct usable as a user attribute (`@Range(min=0.0, max=1.0)` 4052) 4051-4066 | yes | yes | yes | **keep** |
| `@ser` | `skip`, `rename=`, `default`, `since=N` 4076-4116 | yes | yes | yes | **keep** |
| `@soa` | `@soa(flatten)` splits a struct field into columns 3677 | yes | yes, for data-oriented programmers | yes | **keep** |
| `@component` | `@component(layout=soa)` 3815 | yes | yes | yes | **keep** |
| `@layout` | `@layout(c)`, `(c, union)`, `(c, flexible)` 3979, 4532, 4727, 4736 | yes | yes | yes | **keep** |
| `@repr`, `@packed`, `@align`, `@non_exhaustive` | C enum underlying type; packing; alignment; any value may arrive 4724, 4729 | yes | yes | yes | **keep** |
| `@ffi` | one attribute for every C fact: `effects=[...]` 3208, 4593; `deterministic` 3310; `status=, ok=, error=` 3962, 4779, 4879; `link_name=` 4675; `param(...)`, `result(...)` 4639; `macro_fn` 4700; `handle` 4775; `newtype` 4730; `immutable` 4731; `destroys(f)` 4884 | yes: "a fact about foreign code" | jargon, but used only at the C boundary | yes | **keep**; each fact is an assertion, covered by the enclosing `unsafe extern` / `unsafe overlay` |
| `@export`, `@export_table` | C symbol for an Ember function; table of function pointers; `on_panic=abort`, `threads=` 4940-4991 | yes | yes | yes | **keep** |

Must-cover attributes not in this slice (others judge them): `@view`, `@inline`, `@noinline`, `@unroll`,
`@safety`, `@test`, `@should_panic`, `@bench`, `@renamed_from`, `@reinit_on_reload`, `@reloadable`,
`@noreload`, `@allow_reload_terminate`, `@never_specialize`, `@always_specialize`, `@gpu`, `@gpu_layout`,
`@allow`, `@no_derive`, `@hot`, `@cold`, `@grade`, `@deprecated`, `@prelude`, `@no_runtime_checks`,
`@name`, `@dataclass`, `@allocator`, `@borrows`.

## 5. Library words that act as language words

| word | meaning (lines) | same everywhere / every type | plain English | verdict |
|---|---|---|---|---|
| `Box` | one-owner heap value: no `@must_drop` in it 3535; comptime heap value 4031; alloc layer 4129; `Retained.pin` of a `Box` takes ownership, `Box[dyn I]` gives `*mut u8` 4924-4926; no `Box` across an export 4957 | yes | no (Rust jargon; owner rejected `Box[dyn Pool]`) | **merge** into `owned` (JOB: "the one-owner field that replaces `Box` (`mesh: owned Mesh`)") |
| `ForeignBox` | a C object this value owns and destroys through its contract's destructor 4185, 4725, 4826, 4883-4886, 4930 | yes | no, same word as `Box` | **rename** with `Box` (placeholder `Foreign[T]`, held as `owned`); a programmer cares because it is the same idea as `owned` with a C destructor, and the rejected word hides that |
| `Shared` | counted shared ownership, never `Send`, not `Sync` 3441, 3447, 3534, 3738, 4129, 4957 | yes | yes | **merge** with agreed row 9's `shared` (one spelling for one meaning; the builder chooses which) |
| `SyncShared` | shared ownership whose count is atomic, `Send` and `Sync` when `T` is 3441, 3449, 3478, 3535 | yes | inherits the `Sync` problem | **rename** with `Sync` (finding 3) |
| `Weak` | link that does not keep its target alive; `upgrade` is a stale check 3216, 3440-3447, 4957 | yes | yes (JOB: "whether a link keeps its target alive") | **keep** |
| `Cell` | value changed through a shared path by load and store 3366, 3446, 3450, 3588, 3675, 4034, 4170 | yes | no: "cell" does not say "may change though shared" | **keep** for now: not a failed owner test in this slice's uses (no name in a line hides meaning through it); flag for area B/C |
| `RefCell` | borrow checked at run time, one counter update, `requested_by_type` reason 3215, 3226, 3268, 3365, 3446, 3588, 4034, 4195; today's E3023 help offers it (t1) | yes | no: does not say "checked while running" | **rename** (placeholder `RuntimeChecked[T]`): it is the type that costs a check and can panic, and the name says neither; a programmer reading E3023's help (t1) cannot tell what it costs from the word |
| `UnsafeCell`, `Ref`, `RefMut` | 3268, 3446, 4170 | yes | as `RefCell` | **keep**, follow `RefCell` |
| `Span`, `MutSpan` | read-only view; changeable view 3439-3440, 3549, 3576, 3587, 3618, 3663, 3679-3681, 3704, 4207, 4214, 4345-4354, 4648-4666, 4725, 4826-4846 | yes | yes | **keep** the types; **merge** the `mut`-plus-`MutSpan` double (finding 2) |
| `str`, `String` | borrowed UTF-8 view; owned text 4214-4260, 4302, 4332-4339, 4565, 4958 | yes | the capital letter is the only sign of which one owns | **keep**: agreed row 5's message uses `String(...)`; note finding 9 |
| `cstr`, `CString` | borrowed / owned NUL-terminated C text 4220, 4725, 4732, 4866-4877 | yes | as `str`/`String` | **keep** |
| `clone` | `.clone()` keeps the caller's iterator 4378; `Clone` derive 4091; `T: Clone` gathers 3687; `sorted` holds clones 4155, 4322; `comptime(e)` clones 4039 | yes ("a deep copy") | yes | **merge**: agreed row 9 spells a copy `copied items`; `.clone()` stays as the method `Clone` types provide and that `copied` runs, and messages say `copied` (today's E3040 help says "clone the value", t8) |
| `copied` | iterator adapter yielding copies 4367, 4436, 4440 | yes, the same meaning as agreed rows 6 and 9 | yes | **keep**: no clash with the agreed keyword |
| `copy` | `std.mem.copy` (memory copy) 4169; `Copy` marker derive 3712, 4090, 4494, 4532 | yes | yes | **keep** |
| `mem.take`, `mem.replace`, `mem.swap`, `mem.drop` | listed in `std.mem` 4169 (defined in Part IX, reader-2); `Option.take`/`replace` 3901 mean the same | yes | yes | **keep** |
| `Option`, `None`, `Some`, `Result`, `Ok`, `Err`, `?` | 3884-3898, 3876-3877 | yes | yes | **keep** |
| `as_mut` | `Option.as_mut(mut self) -> Option[ref mut T]` 3903; family `get_mut`, `iter_mut`, `values_mut`, `peek_mut`, `chunks_mut` 4286-4315, 4451 | yes | yes ("as" + "may change"; the method says whose: the receiver's) | **keep** |
| `Mutex`, `RwLock`, `MutexGuard`, `Once`, `Atomic`, `MemoryOrder` | 3406-3416, 3448-3466, 3511-3523, 4033-4036 | yes | standard thread words | **keep**; `with g = m.lock():` passes the owner's test through the method names `lock()`/`read()`/`write()` |
| `Retained` | C keeps an Ember object; token whose drop releases or unregisters 4185, 4826, 4900, 4923-4930 | yes ("kept alive for C") | yes | **keep** |
| `ObjectPtr` | address of a class object for C 4926-4928 | yes | yes | **keep** |
| `Pool`, `Handle`, `Entity` | generational handle; 32-bit entity 3811-3813, 4171 | yes | yes | **keep** |
| `Send` | may be moved to another thread 3434-3443, 3504, 3568, 3584, 3628, 3831, 4920 | yes | yes ("can be sent") | **keep** |
| `Sync` | effect 3186, 3230, 3601, 3749, 3757, 4002; type property 3445-3450, 3478, 3568, 3832, 4731, 4966 | **no** (finding 3) | no for the type property | **rename** the type property (placeholder `ThreadShareable`); keep the effect |
| `SoA`, `SoARef`, `SoAMut`, `ArenaSoA` | column storage and its element proxies 3668-3697 | yes | domain term | **keep**; but `SoA.get(i)` → **rename** (finding 6): `get` returns a copy here and a reference on `Array` (4285); placeholder: `get` returns `Option[SoARef[T]]` and the copy is written `copied ps[i]` (or `ps[i].load()`, 3687) |
| `Query`, `Mut`, `Not`, `Commands` | ECS query markers: `Mut[T]` gives `ref mut T`, `Not[T]` excludes 3787-3838 | yes | `Mut[T]` is a second spelling of `ref mut T` | **merge** `Mut[T]` into `ref mut` when phase 2 can (ECS-3 says the language has no access-mode generic, 3826); `Not` keep |
| `Scope`, `JobScope`, `JoinHandle` | 3503-3569, 3626-3629 | yes | yes | **keep** |
| `AnyError` | owned dyn error with context chain 3932-3950 | yes | yes | **keep** |
| `adopt` | `unsafe fn` taking ownership of a C object 4883-4890 | yes | yes | **keep** |

C contract words (overlay and `@ffi(param(...))`, 4819-4851): `borrowed`, `returns_owned(destructor=f)`,
`retained`, `one`, `count(n)`, `nul_terminated`, `fixed(N)`, `inout_count(p)`, `nullable`, `from(...)`,
`exclusive`, `aliased`, `out`, `status`, `handle`, `callback(borrowed | retained | once, user_data=p@N)`,
`threads(...)`, `destroys(f)`, `unsafe`, `TODO(...)`. Each has one meaning, stated in a table that also
names the Ember type it produces. **Keep all**, with one note: `borrowed one exclusive` is written both
`p: ref mut T` and `mut p: T` on the Ember side (4832), the `mut` merge above.

## 6. Today's Ember, pasted (`target/redesign3/bin/ember.exe check`)

t1, `fn integrate(pos: MutSpan[float], vel: Span[float], dt: float):` then `pos[i] += vel[i] * dt` (the
spec's own example at 3576):
```text
error[E3023]: cannot mutate borrowed parameter `pos`
 --> t1_mutspan_no_mut.em:4:9
  |
2 | fn integrate(pos: MutSpan[float], vel: Span[float], dt: float):
  |              ------------------- `pos` is borrowed here
3 |     for i in 0..pos.len():
4 |         pos[i] += vel[i] * dt
  |         ^^^^^^ this write needs mutable access
  |
  = help: restructure to a single owner and declare this parameter `mut pos: MutSpan[f64]`
  = help: if the value is genuinely shared, use `RefCell[T]` for runtime-checked borrows
  = help: if the value has identity, make it a `class`
  = note: `RefCell` adds a one-word runtime borrow-state check in every profile (CELL-5, CELL-9); a class adds a heap allocation, a 24-byte header, reference counting and dynamic exclusivity, and is not `@static_safe` (OBJ-1, RC-1, EXC-1, EFF-13)

error: could not compile due to 1 error(s)
exit=1
```
(Also for reader-4: the help writes `MutSpan[f64]` where the program wrote `float`, and the second and
third helps do not apply to a view parameter.)

t2, the same with `mut pos: MutSpan[float]`: `exit=0`.

t3, `count = 0`; `h = thread.spawn(owned fn() => count + 1)`; `count += 10`; `println(h.join(), count)`:
`exit=0` (the `int` was copied: `count` is still usable).

t8, `name = "ember".to_string()`; `h = thread.spawn(owned fn() => name.len())`; `println(name)`:
```text
error[E3040]: `name` is borrowed after it has been moved out of
 --> t8_owned_fn_move.em:7:13
  |
6 |     h = thread.spawn(owned fn() => name.len())
  |                      ------------------------ moved here
7 |     println(name)
  |             ^^^^ borrowed here after the move
  |
  = help: borrow before the move, or clone the value so the move takes the copy
  = note: the borrow would point at memory whose owner gave it away
  = note: assigning a list, a map or a set moves it; in Python both names would point at one list (Appendix E)

error: could not compile due to 1 error(s)
exit=1
```

t4, `@fp(contract) fn f(...)` and `@static_safe fn g(xs: Span[int]) -> int: return xs[0]`: `exit=0`.

t5, `@sync class Stats` with `total: int` assigned in a method:
```text
error[E7003]: cannot write field `total` of an `@sync` class after `init` or after using `self` as a whole
  --> t5_sync_class.em:11:9
   |
11 |         self.total = self.total + 1
   |         ^^^^^^^^^^
   |
   = help: wrap `total` in `Atomic[int]` for mutation after `init`

error: could not compile due to 1 error(s)
exit=1
```

t6, `p = ps.get(0)` on a `SoA[Particle]`, then `ps[0].life -= 0.5`, then `println(p, ps.len())`:
```text
warning[L1001]: `q` is never read
  --> t6_soa_get.em:12:5
   |
12 |     q = xs.get(0)
   |     ^^^^^^^^^^^^^ declared here and never used
   |
   = help: remove it, or rename it `_q` if that is deliberate

exit=0
```
(The write through `ps[0]` while `p` is still used is accepted, so `p` is a copy, not a borrow of `ps`.)

t7, bare `@nopanic`:
```text
error[E0104]: `@nopanic` is reserved for a later version
 --> t7_nopanic_bare.em:2:1
  |
2 | @nopanic
  | ^^^^^^^^
  |
  = help: write `@nopanic(explicit)`: it forbids the panics the program writes, not every way to stop
  = help: write `@nopanic(explicit)`

error: could not compile due to 1 error(s)
exit=1
```

t9, `return String(word)` (agreed row 5's message spelling):
```text
error[E2020]: `String()` takes no arguments
 --> t9_string_ctor.em:4:12
  |
4 |     return String(word)
  |            ^^^^^^^^^^^^
  |

error: could not compile due to 1 error(s)
exit=1
```

## 7. Error messages seen in passing (reader-4's duty; listed so nothing is lost)

The spec text for these codes in my slice names no line to write instead (51 in all, counting t9's E2020):
E4010 (3207), E4001 (3256), E4002, E4003, E4041, E4042, E4040 (3239-3244), E4070 (3307), E4072 (3332),
E0900 (3300), E7001 (3455), E7006 (3519), E3015 (3535), E3014 (3557), E3021/E3022 (3567), E7005 (3569),
E2071 (3677), E4020 (3756), E7020 (3833), E2180 (3896), W2190 (3919), E2041 (3922), E6005 (4006), E6004
(4008), E6010 (4011), E6001 (4018), E2080 (4103), E2040 (4068, 4498), E2250 (4207), E5063 (4218), E5064
(4220), E2011 (4272), E3063 (4303), E5001 (4624), E5016 (4737), E5017 (4736), E5018 (4738), E5014 (4746),
E5010 (4766, 4853), E5067 (4817), E5052 (4888), E5068 (4890), E5040 (4895), E5041 (4901), E5042 (4914),
E2020 (4950), E1010 (4951), E5015 (4959), E7010 (4966), and today's E2020 for `String(word)` (t9).
Codes reused for two meanings: E5015 and E7010 (finding 11).

## 8. Notes for the builder's annoyance list (not word verdicts)

- Silent copies no word names: `comptime(e)` of a heap value cloned at each evaluation (4038-4040);
  `sorted(m.keys())` clones the keys (4155, 4322); `SoA.get` (finding 6). Area C.
- A lambda written at an adapter call is handed over with its captures (`[CLO-15]`, 4382-4384): the rule
  the owner found wrong for `score` (JOB). Area D.
- `owned fn` callbacks for C (`callback(retained | once)`, 4899) depend on finding 1's replacement. Area D/E.
- `@sync` fields frozen after `init` (3456), with `Atomic`/`Mutex` the only way to change state. Area E.
