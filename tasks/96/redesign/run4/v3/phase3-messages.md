# Run 4 step 2, phase 3: the lead's message fixes

IN PROGRESS. Done: rules rows from areas A, B, C and part of D; areas A, B, C; area D's AN-53, AN-57,
AN-58, AN-61. Still to do: area D's AN-62 (rules E3021 row), AN-67 (E2050), AN-69 (E2041); area E
(AN-71, AN-74, AN-76, AN-77, AN-79, AN-80, AN-84; rules E7007, E7012); the rules' E3023 row (F-06);
the vocabulary's `from` row (F-13).

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
