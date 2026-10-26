# Run 4 step 2, phase 3: the lead's message fixes

DONE. Every message the phase 3 worklist named is fixed: the rule writer's flags (F-01 to F-06, F-11,
the `LOG.retain(keep)` row), areas A to E, the rules' E3023 and E3100 rows and the vocabulary's `from`
row (F-13). Where an area's refused program was not written out, the fix writes it out, so the lines a
message names can be checked against it.

Each row: the failure (`V2/readability.md` F-xx, the rule writer's, or an area updater's flag), where it
was, and what the message says now.

| item | where | fix |
|---|---|---|
| F-01 | area-A AN-01 E3060; rules E3060 row | settled by verdict 42 instead: a `ref mut` name may be given another ref, so v2's line (`boss = ref mut enemies[0]`, `h = ref mut this.handlers.get_mut(i).unwrap()`) compiles and stays; the new-name rewrite was undone |
| F-01 | area-A AN-02, three messages (E3060 on_hit, E3060 kill_weak, E3022 heal_all) | settled by verdict 42 instead: a `ref mut` name may be given another ref, so v2's line (`boss = ref mut enemies[0]`, `h = ref mut this.handlers.get_mut(i).unwrap()`) compiles and stays; the new-name rewrite was undone |
| area A (new) | area-A AN-01 E3060 (`n`) | "keep your own copy: `n = String(...)`" -> "write `n = String(boss.name.trim())` in place of line 8" |
| area A (new) | area-A AN-03 E3022; rules E3022 row | "pass two different bodies, or write a one-body version of `collide`" (words) -> `collide(ref mut bodies[i], ref mut bodies[j])` |
| F-05 | area-A AN-04 E3020; rules E3020 row | `spawned` declared: `spawned: Array[Enemy] = []` before the loop, the push in place of this line, `enemies.extend(spawned)` after |
| F-05 | area-A AN-05, both E3020 | `dead: Array[int] = []` before the loop, `dead.push(i)` in place of this line, the `while` after |
| F-06 | area-A AN-07 E3063 | "make the result `Array[String]` and push `String(w)`" (words) -> the first line `fn words_of(line: StrView) -> Array[String]:`, and in place of line 2 `words: Array[String] = []` then `for w in line.to_lower().split_whitespace(): words.push(String(w))` |
| F-06 | area-A AN-13 E1010 | "write `copy of ps[0]`" (no what, no why) -> names `ps[0].load()`, says a copy is written `copy of`, gives `snapshot = copy of ps[0]` |
| F-06 | area-A AN-16 (`collect`) | `words: Array[StrView] = ...` -> `words: Array[StrView] = line.split_whitespace().collect()`; AN-16's table rows updated (new rows for `first`, AN-07's `words`, `line`, `w`; `snapshot`) |
| F-08 | rules E3028 row | `integrate(ref mut ps, vs, 0.5)` (names no program declares) -> `integrate(ref mut p.as_mut_span(), v.as_span(), 0.5)`, R4's and AN-11's own call (AN-11 itself was rewritten by agent 02) |
| rule writer | rules E3022 row (`LOG.retain(keep)`) | "give `retain` a function that changes nothing: `LOG.retain(fn(s) => s != "")`" (drops the change the program meant) -> `keep` given a concrete body in the row (pushes a note per empty line it drops); way out: `before = LOG.len()`, `LOG.retain(fn(s) => s != "")`, `dropped = before - LOG.len()`, `for k in 0..dropped: LOG.push(String("dropped an empty line"))` |
| F-05 | area-B AN-17 E3020 | the refused `this.add_player(...)` made concrete (`Player(name="imp", hp=3)`, `add_player` pushes); `joined: Array[Player] = []` before the loop, `joined.push(...)` in place of line 17, `this.players.extend(joined)` after |
| F-09 | area-B AN-17 E3021 (another package) | asked the caller to rewrite `note`'s first line in a package it cannot change -> lines for the calling package: `died: Array[String] = []` before the loop, `died.push(f"{p.name} died")` in place of the `note` call, `for msg in died: this.note(msg)` after; `note`'s own package's line kept as a labelled second way |
| F-02 | area-B AN-23 E2140 | "`n = first_value(xs)`, then `n += 1`" (compiles, still loses the change) -> both ways out of verdict 13: `xs[0] += 1`, or `fn first(ref mut xs: Array[int]) -> ref mut int:` with `return ref mut xs[0]`, called `first(ref mut xs) += 1` |
| F-10 | area-B AN-24 E3022 | the view struct `Handle` (a word kept for tokens) -> `Editor`, its list `eds: Array[Editor] = []`; the message gives both changed lines |
| area B (new) | area-B AN-24 E3060 | "read `hs[0].reg.hp[0]` again" (words) -> `println(eds[0].reg.hp[0])` in place of line 3 |
| F-06 | area-B AN-24 E3063 | "keep `reg` alive as long as `vs`" (words) -> the refused function made concrete (`fn views() -> Array[View]:`); `fn views(reg: Registry) -> Array[View] from reg:`, called `reg = Registry(hp=[10, 20])`, `vs = views(reg)` |
| F-06 | area-B AN-25 E3063 | "change `words` to `Array[String]`" -> the field `words: Array[String] = []`, the `init` line `for w in this.text.split_whitespace(): this.words.push(String(w))`, and `this.words.push(String(other_word))` |
| area B (new) | area-B AN-25 E2118 (`buf`); rules E2118 row | "keep the text in a `String`" -> the field line `buf: String` |
| F-06 | area-B AN-31 E2100 | "set `this.renderer` before this line" -> `this.renderer = Renderer(width=width)` before this line |
| area B (new) | area-B AN-22 E2118; rules E2118 row | "rename the parameter" -> both renamed first lines, `... from src:` (the field) and `... from line:` (the parameter) |
| area B (new) | area-B AN-33, three E2104 | "remove `let` from ..." -> the field lines `items: Array[int]`, `name: String`; for `clear` also `fn clear() changes items:` |
| areas B, D (new) | rules E2020 row (AN-56) | `fn(s) => names.push(copy of s)` (changes the captured `names` without saying so, E2105 under verdict 22) -> `fn(s) changes names: names.push(copy of s)`, AN-56's own spelling |
| F-07 | area-C AN-36 L3001 | "link it with `weakref Node` instead" (reads as making the owning children list weak) -> names what would close the cycle, `child.children.push(shares mut child)`; "push each node under one parent only, as `root.children.push(shares mut child)` does"; a back link is `parent: weakref Node = None`; the list stays |
| area C (new) | area-C AN-37 E3022 | "after `path.pop()`, or after the `match`" (where, not a line) -> "write `this.state = State.Idle` below `path.pop()` (line 14) instead of on line 13" |
| F-06 | area-C AN-42 E3020 | "collect what to add in a list, then call `reg.emplace(...)` for each" -> "write `reg.emplace(3, 30)` below the loop instead of in it" (its arguments do not depend on the loop) |
| F-05 | area-C AN-48 E3020 | `new_items` declared: the refused push made concrete (`cam.target.inventory.push(copy of item)`, line 24); `new_items: Array[String] = []` before the loop, `new_items.push(copy of item)` in place of line 24, the `extend` after; or `for item in copy of hud.target.inventory:` |
| area C (new) | area-C AN-48, `cam.shake()` E3020 and (b)'s E3063 | "call it after the loop" -> "write `cam.shake()` below the loop instead of in it"; the field line alone -> `target: shares mut Player` in `Camera` and `cam = Camera(target=shares mut p)` in place of the `sharedref mut` line |
| area C (new) | area-C AN-49 E1010 (`RefCell`) | "write `shares mut Scene`" (a fragment) -> `s = Scene(entities=[])`, the field `scene: shares mut Scene` in each owner, `Panel(scene=shares mut s)` where an owner is made |
| F-04 | rules E3063 kept-view row; area-C AN-52 E3060 and E3063 | `target: shares Player` (refused: the watchers change their players) and field-only lines -> `target: shares mut Player` in `Watcher`, `players: Array[shares mut Player]`, `Watcher(target=shares mut players[r])`; E3060 also "write `players.remove(r)` below line 30"; E3063's positions form: `target: int`, `Watcher(target=r)`, and `players` returned beside `ws` |
| area C (new) | rules E3063 arena row (AN-40) | the caller's lines added, as AN-40's own message has them: `current = LevelArena()`, `boss = load_level(ref mut current)` |
| F-03 | rules E3026 row; area-D AN-57 and AN-53 E3026 | `count = shares mut Counter()` and `count.n` (a class and field the program does not have; the `Button` lines missing) -> every line: `struct Counter:` with `n: int = 0`; in the caller `count = Counter()` and `register(ref mut b, shares mut count)`; `fn register(ref mut b: Button, owns count: shares mut Counter):` (verdict 32); `b.on_click(fn() with owns count changes Counter: count.n += 1)`; in `Button` `handlers: Array[fn() changes Counter]`, `fn on_click(owns f: fn() changes Counter) changes handlers:`, `fn click() changes handlers, Counter:`; AN-57's prose after the message aligned (`with owns count`) |
| F-06 | area-D AN-53 and AN-58 E3020 | "or collect what to add and push it after the loop" (words) -> "call it after the loop, as many times as the loop would: `n = xs.len()` before the loop, and `for k in 0..n: hs[0]()` below it instead of `hs[0]()` in it" |
| F-01 | area-D AN-61 E3060 | settled by verdict 42 instead: a `ref mut` name may be given another ref, so v2's line (`boss = ref mut enemies[0]`, `h = ref mut this.handlers.get_mut(i).unwrap()`) compiles and stays; the new-name rewrite was undone |
| area D (AN-62) | rules E3021 walk row | its "instead of" list left out `if leaf exists as n:`, so following it declared `n` twice (E2106) -> the list names it, as AN-62's own message does |
| area D (new) | area-D AN-67 E2050 | "take the visitor as `ref mut v: V`, or use a type parameter" (a bound, not a line) -> `[V: Visitor]` above `fn make(ref mut v: V)`, or `[I: Inspect]` above `fn walk(xs: Array[I]) -> int:` (verdict 50's spelling) |
| area D (new) | area-D AN-69 E2041 | "narrow the general one's bound so it leaves `i32` out" (no bound can leave one type out) -> `fn describe_int(x: i32) -> String:`, or remove line 9 so that `i32` uses the general one |
| F-11 | area-E AN-74 E7007; rules E7007 row | `with this.lock:` advised for line 19, a task in a free function, where there is no `this` -> an atomic, `total: Atomic[int] = Atomic(0)` in `Stats` and `stats.total.fetch_add(1)` on line 19; or, in `Stats`, `lock: Lock = Lock()`, `total: int guarded by lock = 0` and `fn add_total(n: int) changes total:` with `with this.lock: this.total += n`, and `stats.add_total(1)` on line 19 |
| area E (F-05's kind) | area-E AN-74 E7010 | `hp: int guarded by lock` for a `Player` with no `lock`, and no `with` line -> `players: Array[Player]`; or, in `Player`, `lock: Lock = Lock()`, `hp: int guarded by lock = 10`, `x: int guarded by lock = 0`, and the loop's body inside `with p.lock:` |
| F-05 | area-E AN-76 E7012; rules E7012 row | `FRAME_LOCK` never declared, and `tick` changes `LOG` too -> one lock for both: `static TICK_LOCK: Lock = Lock()`, then `static mut FRAME: int guarded by TICK_LOCK = 0` instead of line 1 and `static mut LOG: Array[String] guarded by TICK_LOCK = []` instead of line 2, and `tick`'s body inside `with TICK_LOCK:` |
| area E | area-E AN-76 `global FRAME` | "today's E0100 help, kept" (not written out) -> the refused program written out, today's text probed (`target/redesign4/lead-phase3/global_frame.em`); "remove line 4 and write `fn tick() changes FRAME:` instead of line 3" |
| F-05 | area-E AN-84 E7012 | `HITS_LOCK` never declared -> `static HITS_LOCK: Lock = Lock()` and `static mut HITS: int guarded by HITS_LOCK = 0`, `bump`'s body inside `with HITS_LOCK:`; or `static HITS: Atomic[int] = Atomic(0)` and `HITS.fetch_add(1)` in `bump` |
| F-06 | area-E AN-77 E2221 (a frame local) | "keep the value instead (`m = n`), or its position" (words) -> the refused program written out; "use `n` itself: remove line 3 and write `yield n` instead of line 5" |
| F-06 | area-E AN-77 E2221 (a linked walk) | "walk it by an index or a `Handle`" (words; a linked list has no index) -> the refused program written out (`Link`, `countdown`); "take the list as a parameter, `gen fn countdown(head: Link) -> Generator[int] from head:` instead of line 5, and remove line 6", as today's help allows a parameter's borrow |
| area E | area-E AN-71 E7007 (float sums) | "`mine += x` for each" (the loop line not given; the program not written out) -> the refused program written out (`total_of`); `scope.spawn(fn() changes sum: sum += p.iter().sum())` instead of line 5 |
| area E | area-E AN-71 E7015 | `cache: thread_shared Cache` alone does not compile (R3: the caller's other owner must be made `thread_shared` too) -> the refused program written out (`share_cached`); `cache: thread_shared Cache` in `Names`, `c = thread_shared Cache()` instead of line 9, `names = Names(cache=thread_shared c)` instead of line 10; the `weakref` variant names `share_plain`'s lines 12 and 13 |
| area E | area-E AN-79 E3100; rules E3100 row | "put it in an `unsafe:` block with a `# SAFETY:` note" (words) -> "above line 4 write `# SAFETY: <why these reads are sound>` and `unsafe:`, and indent line 4 under `unsafe:`" |
| area E | area-E AN-80 E0104; vocabulary `@sync` row | a guarded field with an undeclared lock, and "say which lock guards each field" (words), for a class whose fields are already `Atomic` -> "remove `@sync` (line 1)", with the reason: `Stats`'s fields are `Atomic`, which threads may change as they are |
| F-06 | rules E3023 row (R3) | "or make `game` with `shares mut`" (an action) -> names the refused line, `g2 = shares mut game`, and gives "write `game = shares mut Game()` instead of line 1" |
| F-13 | vocabulary `from` row | C's `from(self)` in the "it will mean" column -> `from(this)`, the receiver's region; today's `from(self)` named as today's |

## After phase 4: the readability check's failures, fixed by the lead

`V3/readability.md` (agent 07) listed 15 failures (its F-01 to F-15, not v2's). The lead checked each
against the files (all 15 hold) and fixed them; `readability.md` itself is left as the checker wrote it.

| item | where | fix |
|---|---|---|
| R-01 (verdict 50) | area-A AN-05 E2113, AN-04 part 3, section 5 item 9; area-B AN-29 E2113, AN-22's refused `put`, section 6 note 7; rules section 0, R5, R12; changes 2.1, section 11 question 4; area-D terms, AN-55 (`inspect`, `spawn`); area-E reading 2, AN-75; vocabulary `[THR-10]` row, `some I` row | `fn name[...](...)` -> `[...]` above `fn name(...)`; the `some I` row now says today's E2260 help becomes `[T: I]` above `fn f(x: T)`. A scan that joins wrapped lines finds no other prose span; what remains is today's Ember, the owner's own sketch, or OLD and "means exactly" code |
| R-02 (verdict 42) | area-B AN-28 part 8; area-A AN-11 part 7, section 5 item 8 | "a `ref mut` name of a place is never re-pointed" -> C++'s two pointers: `ref mut` may be given another ref, `const ref mut` never; "no word says it" -> `const ref mut pos: MutSpan[float]` says it across a package's edge; item 8 marked settled |
| R-03 (verdict 39) | changes 2.1 (foreign calls) | "not on every function that makes a foreign call" -> also on every function that makes an unmarked foreign call, `fn tick() changes Game, LOG:` |
| R-04 (verdict 46) | changes 2.1 and section 3's row; vocabulary iterator row, section 5 | "not made a `dyn Iterator` unless its list is empty" -> the `dyn` type says what it changes (`owns dyn Iterator[Item = int] changes LOG`), refused only when it says less; section 5 now says no word is open (verdicts 46 and 38) |
| R-05 (verdicts 36, 30) | vocabulary `owned fn` row (spec 3495) | `thread.spawn(fn() with owns tx => produce(tx))` -> `thread.spawn(fn() with owns tx changes Sender[int]: produce(tx))`, its caller saying `changes Sender[int]` too |
| R-06 (verdict 40) | area-D summary table and AN-58 part 3 | `Array[fn() changes captured]` -> `Array[fn() changes score]`, as AN-58's code writes it |
| R-07 | rules E3021 row (`blend`) | `assert_disjoint(dst, src)` (the callee's names) -> AN-14's `mix` message: `assert_disjoint(d, s)` before line 12, or the copy lines |
| R-08 | vocabulary `for` row (E3020) | `spawned` declared: the rules' fixed text |
| R-09 | vocabulary `let` row and section 6.2 (E2104), `from` row (E2118), `RefCell` row (E1010) | the fixed answers' lines: `items: Array[int]`; both renamed first lines of `next_word`; `s = Scene(entities=[])`, `scene: shares mut Scene`, `Panel(scene=shares mut s)` |
| R-10 | vocabulary `MutSpan` row (E3028) | `integrate(ref mut ps, vs, 0.5)` -> `integrate(ref mut p.as_mut_span(), v.as_span(), 0.5)` |
| R-11 | vocabulary `Box` row (E1010), `using` row (E3063) | the making line `lvl = Level(mesh=owns Mesh(verts=[0.0]) using current)` added; the E3063 message is the rules' full one, with the caller's `current = LevelArena()` and `boss = load_level(ref mut current)` |
| R-12 | area-B AN-26; area-C AN-35, AN-46, AN-49, AN-50; rules E3060 row | each copy line now says the line it replaces ("in place of line N"); AN-26's and AN-35's refused programs written out so the numbers can be checked |
| R-13 | rules E7015 row | fragments -> AN-78's full message (`main`'s two lines, `setup`'s first line and `return` line, `score: Atomic[i32] = Atomic(0)`, `fetch_add`, `game.score.load()`) |
| R-14 | area-E AN-71 E7014; rules E7014 row; vocabulary `thread_shared` row | `with names.lock:` on a class with no lock -> the refused program written out; "fill the list where the object is made: write `names = thread_shared Names(items=[copy of s])` instead of lines 5 and 6" |
| R-15 | area-B AN-24 E3021 | "change it after `vs`'s last use" (words) -> `reg.hp[0] = 5` below the loop instead of on line 2; the `sharedref mut` way names the field line and both `vs.push(View(...))` lines |

The checker's notes for the lead (its section 7) that were stale text, fixed the same way:

| note | where | fix |
|---|---|---|
| 1 | area-B AN-23, area-D AN-57 and AN-61, their "v3" lines | "the message is v2's / left for the lead" -> what phase 3 did: AN-23's E2140 gives both ways out; AN-57's E3026 gives every line; AN-61's line compiles under verdict 42 |
| 3 | rules R5's caller example | `main` calls a `tick` that changes `FRAME` and `LOG` (R5's own `tick`), so `fn main() changes FRAME, LOG:` |
| 6 | changes section 6 (error codes) | E7018 added (verdicts 43, 52); the E2105, E2113 and E3020 rows name verdicts 36, 37, 39, 45, 46 and 52 |
| 7 | area-E section 4.2 item 3 | the note as put is kept in the past tense, with what the file does since verdict 52 |

Notes 2, 4, 5 and 8 are not stale text: 2 and 4 are open points of the rules (which of E3020 and E7018
comes first for one call; `owns` before a value made `using` an arena), 5 says the `@sync` message
leads to a class that keeps the OLD's race (the NEW is the fix), and 8 lists area A's open rule notes.
