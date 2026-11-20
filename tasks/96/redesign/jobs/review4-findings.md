# Review 4 (pilot, 2026-11-20): soundness of the lead (M6a steps 0-4, A1-17, E7012)

Two lenses at max effort, each finding re-run by a skeptic at max effort. Probes under
`target/scratch/review4/` of the lead worktree. REAL = the skeptic reproduced it.

## positions-across-written-adds

### 0. [REAL] [critical/unsoundness] A written only-adder's result carries the argument's `ref mut` past the skipped conflict: two live `ref mut` names for one element, then use after free
- where: `compiler/ember_analysis/src/positions.rs`:181
- scenario: `grow_and_first` pushes and then returns `ref mut es[0]`. Its summary says it only adds to param 0. So `plan` re-makes `boss` after the call: it skips a call only when `dest` *is* the name, or when the name is one of the arguments (positions.rs:177-183). The borrow check then skips the conflict between the argument's `ref mut enemies` and boss's loan (borrows.rs:4961-4977). That test only asks whether some other holder of *boss's* loan is live at the call. Through the callee's region contract, the argument's own new loan lives on in the result `other`. That never happens with the built-in adds the skip was made for. So after the call, `other` and the re-made `boss` are both live `ref mut` names of `enemies[0]`, and nothing checks one against the other. A view taken through `boss` (`item = boss.loot[0]`) is then freed by `other.loot.clear()`. Expected: E3022 at the call (what c01 gives for `r = ref mut enemies`), or E3060 at the later use of `boss`.
- program: struct Enemy:
    hp: int
    loot: Array[String]

fn grow_and_first(ref mut es: Array[Enemy]) -> ref mut Enemy:
    es.push(owns Enemy(hp=9, loot=[]))
    return ref mut es[0]

fn main():
    enemies = [Enemy(hp=1, loot=[String("a sword with a long enough name")])]
    boss = ref mut enemies[0]
    other = grow_and_first(ref mut enemies)
    other.hp = 2
    boss.hp = 3
    println(other.hp, boss.hp)
    item = boss.loot[0]
    other.loot.clear()
    println(item)
- observed: `ember check` exits 0. `ember run` prints `3 3`, so one element was changed through both names. Then it prints an empty line: `item` reads the String that `other.loot.clear()` freed. A second program, p02_result_keeps_list_run.em, has a callee that returns the whole list (`-> ref mut Array[Enemy]: es.push(...); return es`), and the caller then runs `r.clear(); r.push(...)`. It is accepted, and `boss` then names the element pushed after the clear: it prints `hp: 2 len: 32` and `name: someone else entirely, long text`, and `boss.name = ...` overwrites that element. Controls: c01_control_direct.em (`r = ref mut enemies` with no call) gives E3022. c02_control_not_only_adds.em (the same callee plus a `pop`) gives E3060. Files are in C:/Users/ism19/Code/ember/.claude/worktrees/lead/target/scratch/review4/positions/ (p10_result_aliases_element.em is the program above, plus p01, p02, c01, c02).
- suggested fix: Re-make a position after a call only when the argument's loan cannot outlive that call. In `positions::plan`, skip the call when its result's type can carry a view of that parameter (by the callee's region contract), or when another argument is a `ref mut` of a place holding a view ([LT-47]). Or, in the skip at borrows.rs:4961, also require that no holder of the loan taken at this `ref mut` is live at the call's `next` block.
- skeptic: Reproduced with my own run. This is not a limit the Progress section names. Progress lists `positions::added_arguments` (any argument) as built, and says nothing about a callee whose result keeps the argument's `ref mut` alive. The skip at borrows.rs:4971-4987 only asks whether some other holder of boss's loan is live at the call. Through the region contract, the argument's own new loan lives on in the result, so after the call two `ref mut` names of enemies[0] are live. [BRW-1] ('A `ref mut` name is alone') requires E3022, and the control without the push gives exactly that. positions.rs, summaries.rs, incremental.rs and lib.rs are the same at HEAD as at b8a5133c, so the binary reflects the reviewed code.

### 1. [REAL] [critical/unsoundness] Summaries drop every access made through a view held inside a local struct, closure environment, Option or span: a callee that clears or replaces the list still 'only adds'
- where: `compiler/ember_analysis/src/summaries.rs`:803
- scenario: `reaches` follows a non-argument local only when its path starts with `Deref` (`let Some(Step::Deref) = spot.steps.first() else { return };`, line 803). `pointers_of` records only whole locals of pointer type. So a place reached through a reference stored inside an aggregate local yields no reach at all, not even the `anything` fallback the module doc promises ('so nothing is missed'). Examples: `e.target` where `e = Edit(ref mut es)`, a lambda's environment, an `Option[ref mut T]` bound by a pattern, a span indexed directly. This affects the calls that `mapped` maps, and step 4's `ended_facts` (lines 469-481), whose steps start at the dropped local's field. Effect: `spawn` pushes and then clears `es` through a lambda, a kept-view struct, or a local whose `drop` clears through its view. Its summary still only adds, so `plan` re-makes `boss` and the borrow check skips the conflict. The caller then uses an element whose `drop` has already run. The same hole skips summary_accesses for statics and misses E3020. Expected: E3060 at `boss`'s use, as c04 and c06 give when the clear is written `es.clear()` or goes through a plain `r = ref mut es`.
- program: struct Enemy:
    hp: int
    name: String

    fn drop():
        println("drop runs for", this.name)

fn spawn(ref mut es: Array[Enemy]):
    es.push(owns Enemy(hp=9, name=String("new")))
    wipe = fn(): es.clear()
    wipe()
    println("spawn returns, list length", es.len())

fn main():
    enemies = [Enemy(hp=1, name=String("the boss"))]
    boss = ref mut enemies[0]
    spawn(ref mut enemies)
    boss.hp = 5
    println("boss after its drop:", boss.hp, boss.name)
- observed: `ember check` exits 0. `ember run` prints `drop runs for the boss`, `drop runs for new`, `spawn returns, list length 0`, `boss after its drop: 5 ` (with an empty name). `boss.hp = 5` writes into a dropped slot and `boss.name` reads a freed String. Other programs accepted for the same reason (all in target/scratch/review4/positions/), each run showing the drop before the use:
- p03: `e = Edit(ref mut es); e.target.clear()`.
- p05: a lambda passed to a written `run_it(owns f)`.
- p17: a local `Wiper` struct holding `ref mut es`, whose `drop` clears it when `spawn` ends (step 4's path).
- p09: `d = ref mut es.as_mut_span(); d[0] = Enemy(...)`. `boss` then names the impostor.
- e01: `for ref mut e in es: e.hp = 0` in the callee. `boss`, a live `ref mut`, sees 0 (a [BRW-1] breach).
- s06: a static, `e = Edit(ref mut ENEMIES); e.target.clear()` in `wipe()` while `boss = ref mut ENEMIES[0]` is live. Accepted, and the run uses the dropped element.
- s07: `for line in LOG: wipe()` with that kept-view clear. Accepted, though E3020 is expected; the by-name control c07 gives E3020.
Controls c04, c05 and c06 (direct clear, or a plain local `ref mut`) all give E3060.
- suggested fix: Make `pointers_of`/`reaches` field-sensitive for local aggregates. Follow a struct, tuple, enum payload or closure environment field to the operand it was built from, as `environment_fields` already does for tasks. Treat a leading `Elements` step on a Span-typed local as a step through its pointer. Where a local's contents cannot be followed, fall back to every parameter *and* every module variable (`anything` covers parameters only). `ended_facts` is then fixed through the same function.
- skeptic: Reproduced. Line 803 (`let Some(Step::Deref) = spot.steps.first() else { return };`) drops any place reached through a reference stored in an aggregate: a struct field, a lambda's environment, or a span indexed directly. It does not even fall back to `anything()`, so the module doc's 'nothing is missed' does not hold. The line predates the change (2026-10-30). The change makes it decide verdicts: the summary still says only adds, so the position survives, the conflict at the call is skipped, and step 3's static accesses miss it too. Before the change, any written function here gave E3022 (control f1_c). BRW-16 (a clear or replace ends the position, E3060) and BRW-18 (E3020 through any call that may change the walked list) forbid what is accepted. Progress does not name this limit.

### 2. [REAL] [critical/unsoundness] A view a function returns into a `static mut` carries no facts: a call that clears the static through it is not seen (position E3060, walks, E7012)
- where: `compiler/ember_analysis/src/summaries.rs`:783
- scenario: `fn all() -> ref mut Array[Enemy]: return ref mut ENEMIES`. In `wipe`, `all().clear()` gives its pointer-typed result `Target::Anything`, because the call has no pointer arguments (line 783). `anything()` (lines 831-835) yields only the body's parameters, never a module variable, and the summary records nothing about what a result points into. So `wipe`'s summary has no fact on ENEMIES, and `summary_accesses` makes no access at `wipe()` in `main`. The position `boss = ref mut ENEMIES[0]` survives a clear of its list. [LT-3] requires that a result that may point into a module variable is recorded in the function's summary as pointing into that variable. A push or clear written by the static's name is refused today, as the review asked to check: s01 and c05 give E3060, c07 gives E3020.
- program: struct Enemy:
    hp: int
    name: String

    fn drop():
        println("drop runs for", this.name)

static mut ENEMIES: Array[Enemy] = []

fn all() -> ref mut Array[Enemy]:
    return ref mut ENEMIES

fn wipe():
    all().clear()

fn main():
    ENEMIES.push(owns Enemy(hp=1, name=String("the boss")))
    boss = ref mut ENEMIES[0]
    wipe()
    boss.hp = 5
    println("boss after the call:", boss.hp, boss.name)
- observed: `ember check` exits 0. `ember run` prints `drop runs for the boss`, then `boss after the call: 5 ` (with an empty name), so the dropped element is written and read. The same gap removes E7012. In s09_thread_clears_static_through_returned_view.em, `h = thread.spawn(owns work)` where `work` runs `all().clear()` on `LOG` while main walks `LOG`. It is accepted: a data race on a module variable of the main thread. The by-name control c08 gives E7012. Files are in target/scratch/review4/positions/ (s08 is the program above).
- suggested fix: Record in each summary what its result may point into, both parameters and module variables ([LT-3], [LT-22]). Have `pointers_of` give a call's result those targets instead of `Anything`, and make `anything()` include the module variables wherever it remains a fallback.
- skeptic: Reproduced. In `wipe`, `all()` has no pointer arguments, so its result is Target::Anything (summaries.rs:782-784). `anything()` (lines 831-835) yields only parameters, never a module variable, so `wipe`'s summary has no fact on ENEMIES or LOG. Step 3's summary_accesses and step 0's E7012 therefore see nothing. [LT-3] requires a result that may point into a `static mut` to be recorded in the summary as pointing into it, a reshape while it lives to be E3060, and [THR-20] makes a thread-run function that changes it E7012. Progress names this neither for step 3 nor for step 0 (step 0's not-yet list is only `guarded by`, C callbacks and export threads).

### 3. [refuted] [high/unsoundness] Incremental check keeps a caller whose callee's summary changed: a position skip made against an only-adder outlives a callee edited to clear
- where: `compiler/ember_driver/src/incremental.rs`:858
- scenario: `facts_of` hashes only (region contract, stores) for each callee a kept part uses. The M6a summaries that now decide verdicts (`positions::added_arguments` and the skip in borrows.rs:4961, `summary_accesses`, `reshapes_at`) are not part of it. Step 1: `main` (its own module) holds `boss = ref mut enemies[0]` across `waves.spawn(ref mut enemies)`, which only pushes. Accepted and kept. Step 2: edit only `waves.spawn` to push and then `es.clear()`. The signature, contract and stores stay the same, so `Plan::verify` finds nothing stale and `main`'s borrow check is not run again. The positions pass then reads the new summary, no longer re-makes `boss`, and `boss` keeps the address of the buffer the push freed. The comment at driver lib.rs:5844-5847 claims 'a callee that now removes is then seen'; it is not. The plan lists 'keeping in BorrowFacts' as not yet built, but the consumers are already live in the default `check`, `build` and `run`.
- program: # inc/ember.toml: [package] name = "inc" version = "0.1.0"
# inc/src/main.em
import waves

fn main():
    enemies = [waves.Enemy(hp=1, name=String("the boss"))]
    boss = ref mut enemies[0]
    waves.spawn(ref mut enemies)
    boss.hp = 5
    println("boss after the call:", boss.hp, boss.name)

# inc/src/waves.em, first version (check main.em: exit 0, kept)
pub struct Enemy:
    pub hp: int
    pub name: String

    fn drop():
        println("drop runs for", this.name)

pub fn spawn(ref mut es: Array[Enemy]):
    es.push(owns Enemy(hp=9, name=String("new")))

# then the same file with one line added at the end of spawn:
#     es.clear()
- observed: After the edit, `ember.exe check .../inc/src/main.em` exits 0. With `EMBER_NO_INCREMENTAL=1`, the same command gives `error[E3060]: waves.spawn(ref mut enemies) (line 6) may put another enemy in the place boss names (line 5)...`. `ember.exe run .../inc/src/main.em` (incremental) prints `drop runs for the boss`, `drop runs for new`, `boss after the call: 5 ` (with an empty name): a write into, and a read from, freed memory. The package is at C:/Users/ism19/Code/ember/.claude/worktrees/lead/target/scratch/review4/positions/inc/ and is currently in its edited state. The checks also wrote their incremental records under the repo's target/; none were deleted.
- suggested fix: Make `facts_of` hash the callee's flow summary (`BorrowFacts::summaries`, the facts the consumers read) together with its contract and stores, so a kept caller of a function whose summary changed is checked again. The same applies to E3020 and E3060 through module variables. Until summaries are kept, a kept body must not keep a verdict that read one.
- skeptic: It reproduces, but this is a limit the Progress section already names as not built. Step 1's entry says 'Not yet: the strongly connected groups, bounded facts, keeping in `BorrowFacts`, `[VERIFY-3]` ...'. §1 'Where it is kept' defines that keeping as exactly the missing piece: '`facts_of` (`incremental.rs:857`) hashes (contract, stores, summary), so a kept caller of a body whose summary changed is checked again (`Plan::verify`, `Attempts::again`)'. The risks section says a stale kept fact gives a wrong verdict. incremental.rs is untouched by the change. For the lead: the not-built keeping already has live consequences, because the summary consumers run in default incremental `check`, `build` and `run`. The driver comment at lib.rs:5844-5847 ('a callee that now removes is then seen') is not true today.

## statics-through-calls-and-threads

### 0. [REAL] [critical/unsoundness] A view of a `static mut` returned by a function holds no loan of the static: walks and views of it survive a push or a clear
- where: `compiler/ember_analysis/src/borrows.rs`:6389
- scenario: [LT-3] requires that a function result which may point into a module variable is recorded as pointing into it, and that the caller keeps the variable viewed as it keeps an argument. Its own example (`s = head()`, then `LOG.clear()`, then `println(s)`) must be E3060. Two things stop this. First, `summary_accesses` maps a callee's `Static` facts only onto the root temporaries of statics the body names itself (`runs.statics`), and it returns early when the body names none. Second, a view that comes back from `log()`/`head()` gets the 'static region, so no loan in the caller ties it to LOG. The result: walking LOG through `r = log()` while `tick()` pushes to LOG is accepted. So are `s = head(); LOG.clear(); println(s)` and `s = head(); wipe(); println(s)`.
- program: C:/Users/ism19/Code/ember/.claude/worktrees/lead/target/scratch/review4/statics/p03d_view_from_call_run.em:
static mut LOG: Array[String] = []
fn log() -> ref Array[String]:
    return LOG
fn tick(n: int):
    LOG.push(owns String(f"entry number {n} with a longer text"))
fn main():
    tick(0)
    r = log()
    seen = 0
    for line in r:
        for k in 0..200:
            tick(k)
        println(line)
        seen += 1
        if seen == 3:
            break
    println(LOG.len())
Also p05_head_then_clear_by_name.em (fn head() -> StrView: return LOG[0]; s = head(); LOG.clear(); println(s)) and p04_head_then_clear.em (the same with wipe() doing LOG.clear()), in the same folder.
- observed: `ember check` exits 0 with no diagnostics for p03b, p03d, p04, p05 and p05b. `ember run p03d` printed heap garbage (environment strings, heap metadata) as `line` and ended with "error: the program was stopped by an access violation (0xC0000005)". The step-3 claim in summary_accesses, "A static this body never names holds no loan here", is false: views of statics also flow through function results.
- suggested fix: Build [LT-3]'s rule: record in each summary which statics the function's result may point into, beside the parameters. At the call, give the caller a loan on that static's place that the result holds, making a root for the static even when the caller never names it. Then remove the early return and the names-only mapping in summary_accesses.
- skeptic: This reproduces, and the spec rules it out. [LT-3] says a result that may point into a `static mut` is recorded as pointing into it, and the caller keeps it viewed. Its own example, `s = head()` then `LOG.clear()` then `println(s)`, must be E3060. The compiler does not record this. A view the caller takes itself is refused, so the gap is specifically views that come back from a function. The plan's Progress section names no such limit. Step 0's note 'a caller that never names s holds no loan of it' does not cover it: in s00a, main names LOG itself. The cause is visible in the code: a call result's region sources are only `View{argument}` and `Arena` (regions.rs result facts), with no source for a static. `summary_accesses` also maps facts only onto `runs.statics`. This is new unsoundness, since `static mut` was E1010 before the A1-17 merge.

### 1. [REAL] [high/unsoundness] A change of a `static mut` made through a view the summary cannot follow (a returned view, or one kept in a struct or tuple field) is lost: no E3020, no E7012
- where: `compiler/ember_analysis/src/summaries.rs`:803
- scenario: In `reaches`, the line `let Some(Step::Deref) = spot.steps.first() else { return }` gives no reach at all for a place reached through a pointer stored in a field of a non-pointer local, such as `(*h.items)` or `(*t.0)`. `pointers_of` only records pointers assigned to pointer-typed locals. A pointer returned by a call that takes no pointer arguments becomes `Target::Anything`, and `anything()` (line 831) covers parameters only, never statics. So `fill()` below has no fact for LOG at all: calling it inside a walk of LOG is accepted, and so is `thread.spawn(owns fill)`. The same change made through a plain local view (`r = ref mut LOG`) gets both errors.
- program: C:/Users/ism19/Code/ember/.claude/worktrees/lead/target/scratch/review4/statics/p18_change_through_returned_view.em:
import std.thread
static mut LOG: Array[String] = []
fn log_mut() -> ref mut Array[String]:
    return ref mut LOG
fn fill():
    r = log_mut()
    r.push(owns String("through a returned view"))
fn walk():
    for line in LOG:
        fill()
        println(line)
fn main():
    LOG.push(owns String("start"))
    walk()
    h = thread.spawn(owns fill)
    h.join()
Also p17_view_struct.em: `@view struct Holder: items: ref mut Array[String]` with `h = Holder(items=ref mut LOG); h.items.push(...)`, and `t = (ref mut LOG, 1); t.0.push(...)`, each called inside `for line in LOG:` and spawned on a thread.
- observed: p18 and p17: `ember check` exits 0 with no diagnostics. Control p17b_control.em, the same change through `r = ref mut LOG`: E3020 at line 11 and E7012 at line 17.
- suggested fix: Where `reaches` cannot follow a pointer (a field of an aggregate local, a call result with no pointer argument, the depth cap), count every module variable besides the parameters, or use [LT-3]'s recorded result-to-static facts. Track pointers stored into fields of locals in `pointers_of`.
- skeptic: This reproduces. When a function changes LOG through a view the summary cannot follow, the summary gets no fact for LOG. The cases are a view a call returns, a tuple field, and a `@view` struct field. So the step-3 E3020 for a walk, which Progress claims is built, is missed, and so is E7012. The same change through a plain local view gets both errors. The code shows why. `pointers_of` gives `Target::Anything` to a pointer a call returns when the call has no pointer arguments, and `anything()` covers parameters only, never statics. `reaches` returns at once when a place's first step is not `Deref`, as with `t.0` or `h.items` on a local that is not a pointer. The plan names no such limit.

### 2. [REAL] [high/unsoundness] Generators are invisible to the summaries: a generator body that changes a `static mut` escapes E3020 and E7012, and so does a generator that walks one
- where: `compiler/ember_analysis/src/summaries.rs`:599
- scenario: The borrow checks, and with them `summarize_flow`, run before `coroutines::check_all` and the coroutine transform (driver lib.rs:5790-5861). Until the transform, the checker writes a gen fn's `next` and `drop` as stubs, and its body as a frame method that no call reaches (coroutines.rs:1-22). So a resume (`for v in g`, `g.next()`) and the end of a suspended frame contribute nothing. Four programs are accepted: a generator that pushes to LOG, resumed inside `for line in LOG:`; a thread whose function iterates it (no E7012); a suspended generator holding an Enemy whose drop pushes to LOG, ended inside a walk of LOG; and a generator paused in the middle of `for line in LOG:` while main pushes to LOG by name. That last one is [BRW-18]'s generator clause.
- program: C:/Users/ism19/Code/ember/.claude/worktrees/lead/target/scratch/review4/statics/p13b_generators.em and p13c_generator_thread.em:
static mut LOG: Array[String] = []
gen fn noisy(n: int) -> Generator[int]:
    i = 0
    while i < n:
        LOG.push(owns String("gen"))
        yield i
        i += 1
fn through_generator():
    for line in LOG:
        for v in noisy(2):
            println(v)
        println(line)
fn through_generator_next():
    g = noisy(3)
    for line in LOG:
        v = g.next()
        println(line, v.is_some())
gen fn walker() -> Generator[int]:
    for line in LOG:
        yield line.len()
fn generator_walks_static():
    g = walker()
    first = g.next()
    LOG.push(owns String("pushed while g walks LOG"))
    second = g.next()
    println(first.is_some(), second.is_some())
p13c also has: fn worker(): for v in noisy(2): println(v), then thread.spawn(owns worker); and gen fn holds_enemy(): e = Enemy(); yield e.hp; yield e.hp + 1, used as `g = holds_enemy(); first = g.next()` inside `for line in LOG:`.
- observed: `ember check` exits 0 with no diagnostics for both files. For comparison, a user-written Iterator whose `next` pushes to LOG (p26_user_iterator.em) gets E3020 twice and E7012.
- suggested fix: Treat a call of a generator frame's `next`, and the frame's end, as a call of the frame's body method in call_targets, callees_of, runs_of and drop_runs. Let a frame whose body walks a static keep a view of that static while it is live.
- skeptic: This reproduces, and [EFF-1] requires the checks: 'each `next` carries the effects of the whole body ... and dropping a suspended one carries the drops at every `yield` it may be suspended at'. [BRW-18] also names a generator paused in the middle of a walk. The borrow checks, and with them `summarize_flow`, run before `coroutines::check_all` and the transform (driver lib.rs:5789-5799). At that point the frame's `next` and `drop` are stubs, so resuming or ending a generator does nothing as far as the summaries know. The Progress section does not name this as not built. The Risks section only lists generators among the things to test before M6c.

### 3. [REAL] [high/unsoundness] A function given by value to `scope.spawn` or `js.submit` (a named function, or a lambda that captures nothing) is not followed: no E3020, and no E7012 for a thread that opens a scope
- where: `compiler/ember_analysis/src/summaries.rs`:612
- scenario: For a `FuncRef::Builtin` call, `call_targets` (line 612) and body_facts' task arm (lines 1068-1081) follow only `Operand::Const(Const::Fn)` arguments. With `closure: None` (a named function, or a lambda that captures nothing), lowering passes the task as a callable value, a Copy/Move place. Neither the call's accesses (step 3) nor the caller's summary then includes what the task does. `check_module_variables_on_threads` resolves such places with `targets.of_call`, but only at its own spawn site. Three things are accepted as a result. `scope.spawn(owns note)` inside `for line in LOG:`, where note pushes to LOG. A detached thread whose function opens a scope and spawns `note`, which pushes to LOG off the main thread. The job-scope version of the same. A capturing lambda in the same positions is refused.
- program: C:/Users/ism19/Code/ember/.claude/worktrees/lead/target/scratch/review4/statics/p12_scoped_tasks.em, p12d_scope_in_thread.em, p12e_variants.em:
import std.thread
import std.jobs
static mut LOG: Array[String] = []
fn note():
    LOG.push(owns String("n"))
fn spawn_named_in_walk():
    for line in LOG:
        with scope = thread.scope():
            scope.spawn(owns note)
        println(line)
fn worker_named():
    with scope = thread.scope():
        scope.spawn(owns note)
fn worker_job_scope():
    with js = jobs.scope():
        js.submit(owns note)
fn main():
    LOG.push(owns String("start"))
    spawn_named_in_walk()
    a = thread.spawn(owns worker_named)
    a.join()
    b = thread.spawn(owns worker_job_scope)
    b.join()
(`scope.spawn(owns fn() => note())` behaves the same.)
- observed: p12 and p12d: `ember check` exits 0. In p12e, only the capturing lambda `fn() => note_n(n)` is refused: E3020 at line 16 and E7012 at lines 36 and 40; worker_job_scope at line 38 gets nothing. p12b_scoped_race_run.em (`scope.spawn(owns note)` while main pushes to LOG) also checks clean and crashed at run time with 0xC0000374, heap corruption. That run also needs the next finding fixed before it would be refused.
- suggested fix: For built-ins that run a function argument (ScopeSpawn, JobScopeSubmit, JobSubmit, ThreadSpawn, ParallelFor), resolve Copy/Move callable operands with `targets.of_call(at, body, operand, ty, types)` in call_targets/callees_of and in body_facts' task mapping, as check_module_variables_on_threads already does.
- skeptic: This reproduces. [THR-11] says a task started inside a walk of what it changes is E3020. [THR-20] reaches a scope that a spawned thread opens. The plan's own consumer table (row 'a call's facts as accesses') asks for 'a function-value argument's targets for a built-in that runs it'. But the Builtin arms of `call_targets` and `body_facts` follow only `Operand::Const(Const::Fn)`. A named function or a lambda that captures nothing, given to `scope.spawn` or `js.submit`, is therefore never followed. A capturing lambda in the same position is refused. The plan does not name this as not built.

### 4. [REAL] [critical/unsoundness] A scoped task's accesses to a `static mut` are checked only at its spawn line, not until the scope ends: the block or a second task racing it is accepted
- where: `compiler/ember_analysis/src/borrows.rs`:3960
- scenario: [THR-20] (decision N62) says a static mut the tasks of a scope reach is a place [THR-11] checks as any other. Two tasks changing it is E7007. A task changing what the block reads, or the reverse, is refused, because each task's access lasts until the block ends. The step-3 accesses of a spawn's task are met only by the loans live at the spawn point (the Call arm of check_body), and nothing holds them until the scope's join. Even a capturing lambda, whose facts are seen, races the block: pushing to or walking LOG after the spawn inside the same `with`, or spawning a second such task, is accepted.
- program: C:/Users/ism19/Code/ember/.claude/worktrees/lead/target/scratch/review4/statics/p12g_capturing_race_run.em:
import std.thread
static mut LOG: Array[String] = []
fn note_n(n: int):
    for k in 0..n:
        LOG.push(owns String("from the task"))
fn main():
    n = 200000
    with scope = thread.scope():
        scope.spawn(owns fn() => note_n(n))
        for k in 0..200000:
            LOG.push(owns String("from main"))
    println(LOG.len())
Also p12f_task_outlives_spawn.em: a walk of LOG after the spawn, a push after the spawn, and two capturing tasks.
- observed: p12f and p12g: `ember check` exits 0. `ember run p12g` stopped with "the program was stopped by an exception (0xC0000374)", which is heap corruption.
- suggested fix: Hold a scoped task's (and a scoped job's) static facts from the spawn to the scope's end, the way a closure's captured borrows flow into the Scope. Check them against the block's later accesses (E3020/E3021/E3022) and against the other tasks' facts (E7007).
- skeptic: This reproduces. [THR-11] says what a task changes is checked against the other tasks and the parent until the join. [THR-20] (decision N62) applies that to a `static mut`. A task's step-3 accesses meet only the loans live at the spawn call, so the rest of the `with` block is never checked against the task. Two cases the spec clearly refuses are accepted. One is the parent walking or reading LOG after a task that pushes to it, which [THR-21] also refuses: 'nothing reads the place before the join'. The other is one task writing `COUNT = COUNT * 2 + n` while another reads COUNT, which is not a reduction, so it is E7007. One sub-claim is overstated: two tasks that only push are a [THR-21] reduction, which the spec allows. Even so, the compiler has no reduction lowering (THR-21 appears only in E7007's registry text), so those tasks race as compiled. The plan does not name this as not built.

### 5. [REAL] [high/unsoundness] E7012 misses the drop of a detached thread's result, which the thread itself may run
- where: `compiler/ember_analysis/src/summaries.rs`:311
- scenario: `check_module_variables_on_threads` counts the spawned function's summary plus the drops of the spawn's arguments (lines 311-317), never the drops of the function's result type R. The runtime's `thread_let_go` (ember_rt.c:1630-1636) calls `drop_result` from whichever holder lets go last. When the handle is dropped unjoined first, that holder is the spawned thread. So `R.drop` runs on the other thread, and if it changes a module variable, [THR-20] and [DRP-8] make it E7012.
- program: C:/Users/ism19/Code/ember/.claude/worktrees/lead/target/scratch/review4/statics/p10b_thread_result_drop_run.em:
import std.thread
static mut LOG: Array[String] = []
struct Ticket:
    n: int
    fn drop():
        LOG.push(owns String("pushed by the ticket's drop"))
fn make() -> Ticket:
    total = 0
    for k in 0..30000000:
        total += k % 7
    return Ticket(n=total)
fn start():
    h = thread.spawn(owns make)
fn main():
    start()
    total = 0
    for k in 0..300000000:
        total += k % 3
    println(LOG.len(), total > 0)
    for line in LOG:
        println(line)
- observed: `ember check`: only warning L1001 (`h` is never read), exit 0. `ember run` printed "1 true" and "pushed by the ticket's drop". Main never made or ended a Ticket, so the push came from the spawned thread's drop of its result.
- suggested fix: For ThreadSpawn, add `drop_runs.of(R)` for the JoinHandle's result type to `runs`.
- skeptic: This reproduces at check time, and a run confirms the drop runs on the other thread. [THR-10] says 'Dropping a `JoinHandle` detaches the thread'. [THR-20] says a `drop` that is not plain 'runs where its last owner ends'. The runtime's `thread_let_go` (ember_rt.c:1630-1636) calls `drop_result` from whichever holder lets go last, so a detached thread can drop its own result R. `check_module_variables_on_threads` counts the spawned function's summary and the drops of the spawn's arguments, never R's drop. Step 0's 'Not yet' lists only `guarded by` and the C callbacks and exports.

### 6. [REAL] [critical/unsoundness] Built-ins that run code the program wrote without a visible call (a list or tuple copy runs a written `clone`; `!r` and list formatting run a written `Debug`) are not followed
- where: `compiler/ember_analysis/src/summaries.rs`:594
- scenario: `call_targets` gives a built-in only its `Const::Fn` operands. `ArrayClone {elem}` and `CloneParts {ty}` call each element's written `clone`, and `Format`/`FormatWith`/`Print`/`PrintPiece` call the written Debug writers. `ember_mir::reached_bodies` (ember_mir/src/lib.rs:1668-1681) already lists exactly these edges. So `copy of xs` for an `Array[Leaf]`, or `copy of (Leaf(1), 3)`, where Leaf's clone pushes to LOG, is accepted inside `for line in LOG:`. So are `f"{n!r}"` and `println([n])` when Name's Debug pushes to LOG. `copy of x`, a struct copy, `f"{n}"` and `print(n)` all get E3020.
- program: C:/Users/ism19/Code/ember/.claude/worktrees/lead/target/scratch/review4/statics/p24b_hidden_clone_run.em:
static mut LOG: Array[String] = []
struct Leaf implements Clone:
    value: i32
    fn clone() -> Self:
        LOG.push(owns String("pushed by Leaf.clone, run by the list's copy"))
        return Leaf(this.value)
fn main():
    xs: Array[Leaf] = []
    for k in 0..100:
        xs.push(owns Leaf(1))
    LOG.push(owns String("the first line, pushed by main"))
    seen = 0
    for line in LOG:
        ys = copy of xs
        println(line, ys.len())
        seen += 1
        if seen == 2:
            break
    println(LOG.len())
Also p24_hidden_clone.em (copy_list, copy_tuple) and p25_hidden_format.em (fstring_debug: `t = f"{n!r}"`; print_list: `println(ns)`).
- observed: p24: E3020 only at lines 17 and 23; the list copy (line 29) and the tuple copy (line 35) are accepted. p25: E3020 only at lines 21 and 39; `f"{n!r}"` (line 27) and `println(ns)` (line 32) are accepted. `ember run p24b` printed freed heap bytes in place of `line`.
- suggested fix: In call_targets, callees_of and runs_of, add the bodies `ember_mir::reached_bodies` gives a built-in: clone_dependencies for ArrayClone, CloneParts and copying dyn tables, and debug_writers_reached for the format and print built-ins.
- skeptic: This reproduces. [EFF-1] says an operation's effects include 'the copies ... it performs' and the methods it calls, 'a `Display` a formatter invokes'. `call_targets` and `body_facts` give a built-in only its `Const::Fn` operands. Yet `ArrayClone` and `CloneParts` run the element type's written `clone`, and `Format`/`Print` run the written `Debug` writers. `ember_mir::reached_bodies` (lib.rs:1666-1680) and effects.rs already follow these edges. The plan does not name this as not built.

### 7. [REAL] [critical/unsoundness] A static's run-time initialiser runs at first access but is no call to the analyses: E3020 and E7012 miss what it does
- where: `compiler/ember_analysis/src/summaries.rs`:762
- scenario: `LazyStatic { init, .. }` is treated only as a pointer source (pointers_of, line 762), never as a call of `init`. Under [STA-3], the initialiser of `static CONFIG: int = compute()` runs on first access, on whichever thread gets there first. Two programs are accepted: reading CONFIG inside `for line in LOG:`, where compute pushes to LOG; and a detached thread whose function reads CONFIG, which may run compute and push to LOG off the main thread.
- program: C:/Users/ism19/Code/ember/.claude/worktrees/lead/target/scratch/review4/statics/p19c_static_init_walk_run.em:
static mut LOG: Array[String] = []
fn compute() -> int:
    for k in 0..100:
        LOG.push(owns String(f"pushed by the initialiser of CONFIG, number {k}"))
    return 7
static CONFIG: int = compute()
fn main():
    LOG.push(owns String("the first line, pushed by main"))
    seen = 0
    for line in LOG:
        n = CONFIG
        println(line, n)
        seen += 1
        if seen == 2:
            break
    println(LOG.len())
Also p19_static_init_thread.em (fn reader(): println(CONFIG); thread.spawn(owns reader)) and p19b_static_init_walk.em.
- observed: `ember check` exits 0 for p19, p19b and p19c. `ember run p19c` printed freed heap memory (environment strings) as `line`: the initialiser ran inside the walk and grew LOG.
- suggested fix: Treat `LazyStatic { init }` as a possible call of `init` in call_targets, callees_of and runs_of, so it reaches the summaries, the step-3 accesses and E7012.
- skeptic: This reproduces, and [EFF-1] says it directly: 'a read of a `static` built on first access ([STA-3]) carries its initialiser's effects'. The summaries treat `LazyStatic { init, .. }` only as a source of pointers (`pointers_of`, `runs_of`), never as a call of `init`. effects.rs:1363-1368 does treat it as one. The plan does not name this as not built.

### 8. [REAL] [medium/wrong-refusal] Ending a function value is counted as ending every lambda stored into its type, not the ones that reach its place: wrong E3060 and E7012
- where: `compiler/ember_analysis/src/drop_runs.rs`:148
- scenario: [EFF-2]: "Dropping a function value has the effects of dropping each function that reaches its place: none for a named function, its captured values' drops for a lambda". Only a value that cannot be followed takes every function stored into its type. The `TyKind::Fn` arm of `DropRuns::walk` uses `targets.of_value(ty)`, which is type-wide. With one lambda anywhere in the program that owns a Ticket whose drop pushes to LOG, two things are refused: a lambda that captures nothing, ending inside a walk of LOG; and a thread whose lambda holds nothing.
- program: C:/Users/ism19/Code/ember/.claude/worktrees/lead/target/scratch/review4/statics/p14_fn_value_drop_coarse.em:
static mut LOG: Array[String] = []
class Ticket:
    n: int = 1
    fn drop():
        LOG.push(owns String("ticket"))
fn main():
    LOG.push(owns String("start"))
    t = Ticket()
    keep: fn() -> int = owned fn() => t.n
    println(keep())
    for line in LOG:
        f: fn() -> int = fn() => 1
        println(f(), line)
Also p14b_thread_fn_value_drop_coarse.em: the same `keep`, then `h = thread.spawn(owns owned fn() -> int => 21)`.
- observed: p14: E3060 at line 14. p14b: E7012 "`Ticket.drop` changes `LOG`..." at line 15. Controls p14c_control.em and p14d_control.em, the same files with the `keep` lines removed, check clean (exit 0).
- suggested fix: Pass the ended place to drop_runs and use that place's targets (Targets' place keys, as `of_call` does). Fall back to the type only where the place cannot be followed.
- skeptic: This reproduces, and it refuses valid programs. [EFF-2] says 'Dropping a function value has the effects of dropping each function that reaches its place'. Only a value that cannot be followed takes every function stored into its type. The owner's ODR 15.3 follow-up fix (decisions.md:333) says the same. The `TyKind::Fn` arm of `DropRuns::walk` uses `targets.of_value(ty)`, which targets.rs documents as 'where nothing can be followed'. That is type-wide, though `Targets` has per-place keys (`of_call`). The coarse handling existed before in `effects::drop_of`, but the E3060 and E7012 that come out of it are new in this change. The plan's Risks note on coarse targets is about calls and is not a not-built entry.

### 9. [REAL] [low/wrong-message] An end at a scope exit whose `drop` changes the walked static is reported as "`LOG` does not live long enough" (E3060) instead of [DRP-8]'s E3020
- where: `compiler/ember_analysis/src/borrows.rs`:5224
- scenario: `check_point_with` got a `cause.is_none()` guard on `dropped_temporary`, but not on the [DRP-7] scope-exit branch. A caused access at a `Drop { scope_end: true }` therefore reports that LOG's storage ends: a local of the loop body, a temporary, a struct holding an Enemy, a discarded `pop()` or `remove(0)`, or `mem.drop(e)`. LOG is a static and never ends its scope, and [DRP-8] makes such an end E3020 with the drop's text.
- program: C:/Users/ism19/Code/ember/.claude/worktrees/lead/target/scratch/review4/statics/p09_drop_routes.em:
static mut LOG: Array[String] = []
class Enemy:
    hp: int = 1
    fn drop():
        LOG.push(owns String("enemy gone"))
fn local_ends():
    for line in LOG:
        e = Enemy()
        println(line, e.hp)
- observed: error[E3060]: `LOG` does not live long enough, with labels "this borrow remains live after its source's scope ends" and "`LOG` ends its scope here". Reported at p09 lines 19, 24, 33, 39 and 45, and at p20 line 18 (mem.drop).
- suggested fix: Add `cause.is_none() &&` to the DRP-7 branch so a caused access reaches the walk (E3020) and reshape branches.
- skeptic: This reproduces. The refusal is right, but the code and the text are wrong. [BRW-18] lists 'the end of an object whose `drop` changes it, [DRP-8]' as E3020, and [DRP-8] includes 'a local ending with its function'. Here the compiler gives E3060 and says the static LOG 'ends its scope here', which is false. In `check_point_with`, the `dropped_temporary` branch got a `cause.is_none()` guard but the [DRP-7] scope-exit branch (borrows.rs:5219-5242) did not, so a caused access at `Drop { scope_end: true }` takes the lifetime message. Progress claims the walk texts for DRP-8 are built. Low severity.

### 10. [refuted] [low/spec-deviation] A class-typed `static mut` is not one place at compile time: a walk plus a push by name passes, where a local class object gets E3020
- where: `compiler/ember_analysis/src/borrows.rs`:5639
- scenario: [LT-3] says a function naming a static mut reaches it as it reaches a local of its own. For `static mut G: Game` (a class), `for line in G.inner.b: G.inner.b.push(...)` passes `check`. The same with a local `g = Game(...)` is E3020. Only the run-time exclusivity check stops it. Likely cause: each use reads G's handle into its own temporary, which `module_variable_place` (Deref-led places only) does not map to one root.
- program: C:/Users/ism19/Code/ember/.claude/worktrees/lead/target/scratch/review4/statics/p16c_class_static_run.em:
struct Inner:
    a: Array[String]
    b: Array[String]
class Game:
    inner: Inner
static mut G: Game = Game(inner=Inner(a=[], b=[]))
fn main():
    G.inner.b.push(owns String("start of the walked list"))
    for line in G.inner.b:
        G.inner.b.push(owns String("x"))
        println(line)
Compare p16b_class_local.em.
- observed: p16 and p16c: `ember check` exits 0. p16b (local class): E3020 at line 14. `ember run p16c`: "panic ... exclusivity violation: write access to Game.inner while a read access to Game.inner is active". This is safe at run time but not refused when compiling.
- suggested fix: Map every handle temporary read out of one class-typed static to that static's one root, as is done for `(*_t)` places.
- skeptic: Not a defect of this change. The compile-time miss reproduces, but a run shows it is memory-safe: the run-time exclusivity check stops the program with a panic. It is also the class-object limit the plan names as not built ('class objects checked while running, not when compiling', steps 12-13, which remove keep_handle_alive and add the may-be-one query through handles). Each use of G reads its handle into a temporary of its own. The same miss happens without any static: a local class whose field is a class (s10e) is also accepted at compile time, so it is not specific to statics.

### 11. [REAL] [low/wrong-message] The new chain and end texts name compiler internals: a mangled lambda symbol, a temporary, standard-library line numbers, and a method without its type
- where: `compiler/ember_analysis/src/borrows.rs`:303
- scenario: [DIA-2] and [DIA-27]. `body_names` uses `body.name` for lambdas. The End cause quotes the Drop's place even when it is a temporary (borrows.rs:6474). `program_chain`/`in_std` do not treat instances of std generics as std, so the chain stops inside std and gives a std line. E7012 names an `extend` method by its bare name.
- program: In C:/Users/ism19/Code/ember/.claude/worktrees/lead/target/scratch/review4/statics/: p08_std_callbacks.em (`xs.retain(owns fn(x) => note())` in a walk of LOG); p20_more_ends.em (`c = Cell(Enemy())`; `c.set(Enemy())` in a walk); p01_display_walk.em (`println(n)` with a written Display that pushes to LOG); p23_hidden_compare.em (`xs.sort()` with a written Ord); p26_user_iterator.em (thread.spawn of a function using Count.next).
- observed: "(through `em_closure_13retain_lambda_c0` and `note`, line 4)"; "`c.set(Enemy())` (line 14) replaces `_17`, which ends its `Enemy`"; "(through `println`, line 174)"; "(through `sort_ord`, line 2594)"; "`next` changes `LOG`... `static NEXT_LOCK`" where the spec names a method by its type (`Count.next`, COUNT_LOCK).
- suggested fix: Name lambdas as written; describe a temporary by its expression; count std instances as std in program_chain; use owner_of for E7012's name.
- skeptic: This reproduces. [DIA-2] requires a refusal to name things 'in the program's own names (`line.to_lower()`, not a temporary's internal name ...)', with internal names only as secondary evidence. [DIA-27] requires the chain of the program's calls. [THR-20] and Progress step 0 say 'a method by its type (`Ticket.drop`, `TICKET_LOCK`)'. Every one of the four kinds the finding lists shows up: a mangled lambda symbol, a temporary, standard-library line numbers and function names, and a method named without its type. The same also happens for a struct's `drop`. Low severity.

### 12. [refuted] [high/unsoundness] After 207ee285, a `static mut` walked through a parameter while the callee changes it is accepted (the E3031 call check that should replace the old refusal is not built)
- where: `compiler/ember_analysis/src/borrows.rs`:3965
- scenario: 207ee285 is outside the three named commits but inside `git diff f503f014`. It makes a call's caused accesses meet only the loans still used after the call (the `across` filter), and leaves argument loans to E3031 (step 6, not built). In `walk(LOG)`, walk's loop walks its view parameter while `tick()` pushes to LOG. The 15:40 build refused this with E3021 at `walk(LOG)`. The current target/ci/ember.exe (built 15:53) accepts it. All other results in this report were re-checked with that binary.
- program: C:/Users/ism19/Code/ember/.claude/worktrees/lead/target/scratch/review4/statics/p03c_param_walk.em:
static mut LOG: Array[String] = []
fn tick():
    LOG.push(owns String("tick"))
fn walk(xs: Array[String]):
    for x in xs:
        tick()
        println(x)
fn main():
    tick()
    walk(LOG)
Also p03e_param_walk_run.em: 200 pushes per element, then println(x).
- observed: p03c and p03e: `ember check` exits 0 (before 207ee285, p03c gave E3021 at line 13). `ember run p03e` printed the element through its view after 200 pushes had reallocated LOG (LOG.len() 201). That read freed memory; the bytes happened to still be intact, so the output looked normal.
- suggested fix: Keep argument loans in the caused-access check until check_call_summary (E3031) lands, or land step 6 together with this narrowing.
- skeptic: It reproduces, but it is a limit the plan names as not built. The committed Progress section's 'Step 6's plan' is the call check `check_call_summary`: each argument's loans against the callee's `used_after(i)`, E3031. That is exactly what refuses `walk(LOG)` when walk walks its parameter while `tick()` pushes. 207ee285's own commit message says argument loans are now left to E3031 (step 6). Still, the owner should know this is a deliberate soundness regression until step 6 lands: b8a5133c refused this program.

## The fix plan (the lead, 2026-11-20)

The lead is not merged to `main` until every REAL finding is closed. `main` refuses `static mut` (E1010), so it
has none of these. The rule for each: an analysis that cannot follow something counts it as reaching anything
(sound), and a route a later milestone builds is refused with `E0900` until then.

1. **Facts the summaries cannot follow** (statics 1, positions 1): `summaries::reaches` and `pointers_of`
   over-approximate. A pointer read out of an aggregate's field (a struct, a tuple, an `Option`, a closure's
   state, a span), a call's result with no pointer argument, and the depth cap reach every parameter AND every
   module variable (a `Root::AnyStatic` fact: a walk or a view of any static meets it; `E7012` for a thread
   that may run it). Pointers stored into a local aggregate's fields are followed where they can be.
2. **A view a function returns into a module variable** (statics 0, positions 2): `[LT-3]` is not built (the
   result's region has no static source): `E0900` at the function's `return` of a view that may point into a
   `static mut`, until LT-3 (M5's left part).
3. **Code a built-in runs** (statics 6, 7): a static's first-access initialiser (`LazyStatic { init }`), a
   written `clone` a copy runs, a written `Debug`/`Display` formatting runs (`debug_writers_reached`): call
   targets in `summaries::call_targets`, as `effects.rs` counts them.
4. **Functions given by value** (statics 3): a function-typed operand given to a built-in (`scope.spawn`,
   `js.submit`) is followed by `Targets::of_call`, as a `Const::Fn` one is.
5. **Generators** (statics 2): a generator's resume runs its body: the body is a call target at each `next`.
6. **Scoped tasks and module variables** (statics 4): `[THR-11]` for a `static mut` is M9's: a scoped task or a
   scoped job that reaches a module variable is `E0900` until then.
7. **A thread's result** (statics 5): `E7012` adds the `drop`s of the thread's result type.
8. **Positions across a written add** (positions 0): a call whose result may hold a view into the list (its
   result's regions) keeps no position across it.
9. **Ends of a function value** (statics 8, a wrong refusal): the targets of the place, not of the type.
10. **Messages** (statics 9, 11): a scope exit's end gets DRP-8's E3020; no mangled names, temporaries or
    standard-library lines in a chain.
11. **The argument's own view** (found while fixing, p03c): 207ee285 let a call's caused accesses skip the loans of
    views whose last use is an argument, trusting the call check (E3031) that step 6 builds; until then that
    skip accepts `walk(LOG)` whose callee walks its parameter and pushes to LOG. Reverted: the accesses meet the
    arguments' loans too. Seven accepting files wait for step 6 in `ahead/M6a` again: BRW-8 (2), BRW-17, BRW-21
    (3), MONO-10.

Done 2026-11-20: 1 (views kept in a local aggregate are followed; one nothing records reaches anything), 2
(E0900), 3, 4 (only for the built-ins that run what they are given), 7, 8, 11. Left: 5 (generators), 6 (scoped
tasks racing on a module variable), 9, 10.
