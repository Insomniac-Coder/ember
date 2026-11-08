## K10-queued-drop-check-points (P05; J7-01, J7-02): **rejected**, 11 agents

Outcome: `decided_by_parameters`. Intent: A thing one thread made and another let go of (a pooled connection, a GPU texture, a packet buffer) is ended soon by its drop, at a point where running that drop breaks no rule, and is never left undone because its making thread sleeps in the job pool.

Where the cause is: RC-4 point 3 (tasks/spec-0.9.9/parts/p08-classes.md 169-172), with RC-4 point 4 (p08 173-178), THR-20's headline 'A `drop` runs where the last owner ends' (p11-concurrency.md 296-301) and DRP-8 (p07-ownership.md 865-873). (reported at: BRW-18, EFF-24, FFI-22, HR-3a, JOB-2, RC-3, RC-4, THR-16, THR-19, THR-20, WK-3). Why there: A check point is where a queued value is ended, so it is where its drop runs (RC-4 point 2, p08 166-168; WK-3, p08 365-367). DRP-8 makes every end a call of the drop 'for every rule that needs them'. Point 3 places those ends by position only. So they land inside a lock the drop takes, inside a walk or view of what it changes, inside a scope whose tasks touch it, or inside a no-waiting function (J7-01). It also ties the last end to the thread's end, which a pool worker reaches only with the process (THR-16) and a host thread never shows (FFI-31a) (J7-02). The debate showed the same cause three more ways. (1) A value whose maker can never run its drop safely must not be queued at all, and that is judged by where its making line runs, not by where it is written (point 4). (2) A by-type view rule is untrue for what the queue ends only for a view that holds a count or reads through an owner. A `weakref` view of an object that is not `thread_shared` does neither. (3) A thread's last point must come before its join is let go. Then THR-20's count (the making thread may run the drop) stays true and THR-19 judges the join. The reporting rules are right as written: each one sees only what point 3 and THR-20 hand it.

**In short: A deleter owed to a scoped worker thread (a shared_ptr whose destructor must run on the thread that made it) needs a mutex the parent holds while it waits for the scope to finish.**

step | who | what happens | result
1 | scoped worker thread | opens a connection and puts it in an inbox | about to finish
2 | closer thread | lets go of the connection | its destructor is owed to the scoped thread
3 | main | holds the pool's mutex and waits for the scope to finish | waiting
4 | scoped thread | runs the owed destructor before finishing; it needs the pool's mutex | each waits for the other
5 | the changed rules | one says refuse main's wait, another says compile it | two answers for one program

Today: Not said whether a scoped thread's last owed destructors run before or after the scope's wait ends.

* a: count a thread's last owed destructors as part of what that thread does, scoped threads too; the wait under the mutex is refused with a one-line fix (wait after the mutex block)
* b: run a scoped thread's last owed destructors after the scope's wait ends; it compiles, but the scope's end no longer means they are done
* c: leave the words as they are: one rule refuses it, another compiles it and it freezes

Recommendation: a: one answer, decided when compiling, nothing added while running, the same as a joined thread already gets.

Arbiter: One open objection blocks: E3-1, a contradiction. The change says a thread's last pending destructors run before its join or its scope's end lets go, and that they count as part of what the thread does. But the only rule it cites for that count covers spawned threads and unscoped jobs, not scoped tasks, and it must stay that way, or the change's own loader tests become errors. So a scope's end inside a lock, whose task may end such a value at its last point, is refused by one rule and compiled by another, and when compiled it hangs. The other safety claim, S3-1 (a use after free through the point after a chunk main runs itself), is refused. Read one way, the placement rule keeps that point out. Read the other way, the base destroy-counts-as-a-call rule refuses the program. Either way nothing reads freed memory or hangs. The wording should still say which. E3-2 (prebuilt libraries) is refused: the rule is a whole-program property, and the per-thread number already listed covers it. E3-3 and E3-4 are real but do not block. The cost figures shown to the owner misread the measured table, and decision 4's option b needs its scope and cost stated. The change goes back to the designer for one more round on E3-1. J7-01 and J7-02 themselves stay decided.

### Edit: RC-4 point 3 (change, tasks/spec-0.9.9/parts/p08-classes.md 169-172)

**Before:**

>   3. **Check points.** The compiler places them: at every loop's back edge, before every call that may
>      block the thread (waiting for a lock, `join`, a channel's receive, a sleep) and when the thread
>      ends. A value whose making thread has ended is ended at once by the thread that ends its last
>      owner.

**After:**

>   3. **Check points.** The compiler places them: at every loop's back edge, before every call that may
>      block the thread (waiting for a lock, `join`, a channel's receive, a sleep), when a call C made
>      into Ember returns to C, after each job a thread runs (a `@parallel` loop's chunks included) once
>      its end is recorded for what waits for it, and when the thread ends, before what waits for it (its
>      `join`, its scope's end) is let go; a job worker waiting for work waits at one, and a value put in
>      its queue wakes it. Emptying the queue ends, on this thread, the values whose counts have all
>      ended (point 2), and their `drop`s run there: it is a call of the `drop`s of every value that may
>      be in this thread's queue (`[DRP-8]`), with the queue's own count changes. A check point in a
>      function's lines is placed only where that call would compile if it were written there, judged
>      for each place the function runs from (its callers; a task's body against its parent and the
>      other tasks until its join, `[THR-11]`), and where this thread leaves the last place that keeps
>      one out (the end of a lock's `with` block or guard, of a walk, of a view, of a `with scope` block,
>      the return from a function with a contract) is a check point. A thread's last check point is
>      always placed: for a thread other than the main one, what it may end there is counted in its
>      function (`[THR-20]`), so every wait for the thread is judged with it (`[THR-19]`, `[THR-11]`).
>      What the queue ends, the values and what only they own, has no owner left, so a view that reads
>      its object through an owner or a count of this thread (`[RC-5]` point 2) cannot be reading it,
>      and a rule that judges by type what a call may end (a view's, `[RC-5]` point 1) does not keep a
>      check point out for such a view. A view opened from a `weakref` link to an object that is not
>      `thread_shared` holds neither, so it, and every view made through it, keeps check points out for
>      its life as that rule says. A value whose making thread has ended is ended at once by the thread
>      that ends its last owner.

Why here: Point 3 is the only place where hidden ends are placed. The same words decide J7-01 (no point inside `with pool.lock:`, and the block's end is one) and J7-02 (an idle worker is woken). They give the bound the conflict note asks for, and they decide the debate's scope-task, walk, view, weakref-view and join programs.

Why general: One property covers every hidden end: the queue is emptied only where a call of the queued drops would compile if it were written there. So every rule that judges a call judges this one (THR-19 in full, BRW-18, RC-5, THR-11 against a scope's tasks, every contract, THR-20), with no list of rules. Where the thread leaves the last place that keeps a point out, there is a point. That one phrase is the bound for every kind of place. The view exception is now true for what it covers. What the queue ends can be reached by this thread only through a `weakref` link. A view opened from one holds a count when its target is `thread_shared` (RC-5 point 2, p08 215-219). Every other view reads through an owner. So the exception holds for those views, and a `weakref` view of a plain class object, which holds no count, stays under RC-5 point 1 as written. The new points cover every kind of thread: spawned, main, job worker, and a thread C calls in on. A thread's last point comes before its join is let go, so `join` means done, and THR-20's count lets THR-19 judge the join. A job's point comes after its end is recorded, because a worker's queue holds values from any earlier job. No type or pattern is named.

### Edit: RC-4 point 4 (add, tasks/spec-0.9.9/parts/p08-classes.md 176-178 (sentences added after 178))

**Before:**

>      first adds its own count of the value into the away count and marks the value handed over; from
>      then on every thread, the making thread included, counts in the away count, and a later end on
>      another thread never goes through the queue.

**After:**

>      first adds its own count of the value into the away count and marks the value handed over; from
>      then on every thread, the making thread included, counts in the away count, and a later end on
>      another thread never goes through the queue. A thread also counts a value this way from the line
>      that makes it when its own check points may not run that value's `drop` until a time that may
>      never come, judged for each place that line runs from, through any chain of calls (`[EFF-24]`):
>      the line runs inside a function whose contract forbids an effect of the `drop`, which may never
>      return; or it runs on the thread of a task or a job of a scope, or of a chunk of a `@parallel`
>      loop, which may go on beside the main thread after the scope, and the `drop` reaches a module
>      variable of the main thread, which no other thread may reach (`[THR-20]`). Its `drop` then runs
>      where its last owner ends.

Why here: Point 4 is the rule that moves the making thread's count into the away count, so a value that must not wait for its maker belongs there.

Why general: One property: a value whose maker's own check points may never run its drop is counted for every thread from the start. Its drop then runs where its last owner ends, which is where THR-20 and DRP-8 already judge it. It is judged by where the making line runs, through any chain of calls and for each place, as point 3 judges its points. So a constructor helper called from a job (`load_texture(path)`) is covered just as a making line written in the job's body is. The parenthesis names the two kinds of place, as point 4's own parenthesis does. They are the only places that can hold a drop back for good when another thread could run it. A lock, a walk, a view or a scope would hold up any thread just the same.

### Edit: THR-20 (add, tasks/spec-0.9.9/parts/p11-concurrency.md 296-297)

**Before:**

> **A `drop` runs where
>   the last owner ends.** The last owner of a `thread_shared` value may end on any thread that holds a

**After:**

> **A `drop` runs where
>   the last owner ends**, or, for a value queued back to the thread that made it (`[RC-4]` point 2), at
>   a check point of that thread, placed where the `drop` may run (`[RC-4]` points 3 and 4): the function
>   of the thread that makes such a value holds a count of it from the making line to the thread's last
>   check point. The last owner of a `thread_shared` value may end on any thread that holds a

Why here: This rule judges which thread a drop runs on, so it must name both places where a drop runs. Here E2-1's contradiction is closed: the joined thread's function counts the drop, and nothing elsewhere takes that away.

Why general: Without the clause, the headline is untrue for every queued value. The clause also says plainly what the next sentence ('a function that may run on another thread and holds such a count counts as possibly running that value's `drop`', p11 298-301) needs: the maker holds its home count until its last point. So E7012, THR-13's module-variable bullet, THR-11 and THR-19's join and scope-end clause all see the drops a thread may run before it is joined.

### Edit: DRP-8 (add, tasks/spec-0.9.9/parts/p07-ownership.md 873 (a sentence added after it))

**Before:**

>   A `drop` may change things outside its object (a log line, leaving a list).

**After:**

>   A `drop` may change things outside its object (a log line, leaving a list). The ends at a check
>   point placed in a function's lines (`[RC-4]` point 3) are judged by that rule where it places them,
>   which is only where they break no rule; they are not part of that function's effects (`[EFF-1]`) or
>   summary (`[EFF-24]`) beyond what `[THR-20]` counts for a function that holds a count of the value,
>   which also covers a thread's last check point.

Why here: EFF-1 (p10 26-27), EFF-24 (p10 96), BRW-18 (p07 508) and RC-5 point 1 (p08 202) all route drops through DRP-8.

Why general: Without it, EFF-1's 'the drops it performs' would put every queued drop into the effects and summary of every function that loops. Every `@noblock` loop would be E4003, every walk across a call to a looping function E3020, and every task with a loop E7007. That would be untrue, because placement already keeps those ends where they break no rule (principle 6). The sentence takes away nothing that THR-20 counts, so the two cannot decide one program in different ways. BRW-18, RC-5 point 1, EFF-1 and EFF-24 all reach a drop through DRP-8, so this one sentence reaches all of them.

### Edit: COST-3 (the row 'A `thread_shared` value's two counts') (add, tasks/spec-0.9.9/parts/p10-effects.md 304)

**Before:**

> a test of the thread's queue at each check point (`[RC-4]`), and the queue's work for a value whose last owner ends on another thread without a hand-over.

**After:**

> a test of the thread's queue at each check point (`[RC-4]`), and the queue's work for a value whose last owner ends on another thread without a hand-over, with one wake of its making thread when that is a job worker waiting for work; atomic counts from the start for a value made where its making thread's check points may not run its `drop` (`[RC-4]` point 4); and, for a function whose check points or making lines are judged differently for the places it runs from, one more copy of its code or, where the compiler keeps one instead, a plain add and subtract of a per-thread number at each such place, which its check points and making lines test (`[RC-4]` points 3 and 4).

Why here: This row lists the run-time costs of the two-count design.

Why general: COST-3 lists everything that runs, and COST-5 (p10 353) asks for a row for each new implicit cost. The new costs are the wake, the hand-over at the making line, and the copy or the per-thread number, which now also serves making lines. The copy is named beside the number, as p10 343 names BRW-11's copy.

### Impact

Rules changed: RC-4 point 3 (p08 169-172), changed. Points are placed only where a call of the queued drops would compile. Where a thread leaves the last place that keeps a point out, there is a point. There are new points after each job, at a callback's return to C, and for a worker waiting for work. A thread's last point comes before its join is let go. The view exception now covers only views that read through an owner or a count. RC-4 point 4 (p08 178), added: a value is counted for every thread from its making line when its maker's points may never run its drop. This is judged by where that line runs. THR-20 (p11 296-297), added: a queued value's drop runs at its maker's check point, and the maker's function holds a count until its last point. DRP-8 (p07 873), added: ends at points inside a function are judged where placed and are not in the function's effects or summary, beyond what THR-20 counts. COST-3 (p10 304), added. Not changed: RC-4 points 1, 2 and 5; WK-3; RC-5; THR-11; THR-13; THR-19; BRW-18; EFF-1; EFF-24; RC-3; JOB-2; HR-3a; FFI-22; THR-16.

Readability: Better, with nothing added. The one visible difference: a queued drop's output may appear a little later, where the thread leaves the place the drop would have broken.

Cost: Compared with C++ `shared_ptr` and today's Ember (C has no automatic end). Nothing changes per count operation for a value whose maker can run its drop. A check point is still one thread-local load and a branch. New tests come after each job, at each return to C (beside FFI-22's attach check), and where a thread leaves each place that keeps points out; back edges inside those places lose theirs. A sleeping worker is woken (one system call) only for the first value put in its queue. Atomic counts from the making line, at most the measured 9% to 13% of objects passed between threads, apply only to values made under a contract that forbids their drop's effect, or in a scope's task, job or chunk, with a drop that reaches main's module variables. A function judged differently at different call places costs a copy (code size, COST-2) or one plain add and subtract of a per-thread number per place. Ordering a thread's last point before its join costs nothing. To measure on the laptop's P-core, for speed and memory: C backend with MSVC and with Clang, and the LLVM backend.

* verdict 101: the making thread deletes at its next check point, placed by the compiler, always before the thread ends; or the other thread deletes at once if the maker has ended: kept (A queued value is still ended only by its maker, at a point the compiler places, always before the thread ends, and now also before its join is let go. The ended-thread path is word for word the same. Counting a value in the away count from its making line is fix 2's step, taken where the compiler sees the maker can never end it. The arbiter should judge whether that stays within fix 2 (decision 3). Decision 4's option b would widen the ended-thread path, so it goes to the owner and is not applied.)
* verdict 43: a call that may take a lock this thread holds is refused when compiling, nothing checked while running: kept (A hidden end never waits for a lock its own thread holds; placement decides that when compiling. A join under a lock that the joined thread's drops take is refused (E7018), as verdict 43 refuses a call.)
* verdict 57: destroying an object counts as calling its drop: kept (Emptying the queue is a call of the drops, judged at every place it is put. It stays out of a function's summary only because there it would be untrue: the copy called inside a walk of `LOG` runs no such drop. THR-20's count is kept. For the arbiter: this is the one place where verdict 57's 'a function that may destroy an Enemy changes LOG' is not carried into a summary.)
* verdict 104: a scoped task's claims end at its join: kept (A point inside a scope, or in a task's body, is judged against the tasks not yet joined. The task's last point comes before its join, and is judged with the task.)
* verdict 56: an object counts as ended before its drop runs: kept (Unchanged.)
* verdict 114: a moved object's drop runs once, on the other thread: kept (A moved one-thread object is never queued.)
* verdict 86: what a function changes is worked out, never written: kept (Points, and which values are counted from their making line, are worked out from the drops' code.)
* verdict 88: the compiler makes a copy of a function where callers differ: kept (Used for a function whose points or making lines are judged differently at different call places.)
* verdict 122: by field, not by type: kept (For what the queue ends, a view's by-type rule gives way to the object only where the view reads through an owner or a count. Where it cannot, the by-type rule stands.)
* a program changes: Not decided. The inner walk's back edge may run Conn's drop there: it would wait for the lock main holds, or push into the walked list. -> Compiles. The drop runs where the `with` block ends. Later frames print 1, and the join prints 1.
* a program changes: The Textures are never freed while the game runs. -> Each worker is woken and prints 'texture N freed' once, right after the clear.
* a program changes: E3060 at `p.data[i]`, or not decided (the last round's text: a use after free). -> Compiles. No point inside the view's life. The Conn is ended after `p`'s last use. Prints `1 1 24`.
* a program changes: The Textures wait for ever in the idle workers' queues: prints 1000, or more if the joiner ran a job (the last round's text: a race between the woken worker and main). -> Counted for every thread from the making line, judged for the job's place. `bundles.clear()` ends the Textures on main. Prints `1002`.
* a program changes: E7018 (THR-19 with THR-20). Under the last round's text it would also compile and print 0. -> E7018 at `h.join()`, as today, with the fix 'join after the `with pool.lock:` block'.
* a program changes: Not decided whether the drop has run when `join` returns: prints 0 or 1. -> Prints `1`.
* a program changes: E7007 or E3020 if DRP-8 counts main's back edges; otherwise a race or a push into the walked list. -> `499999500000 1001`; `10 5`; `start`, `enemy gone`, `2`; `499500 2`.
* a program changes: Prints 1000, 1001 or 1002; a race once a later job runs on that worker. -> Prints `1002`.
* a program changes: E4003, or the drop runs inside `@noblock poll`. -> Prints `1000 buffers back`.
* a program changes: Not decided: E4003, or the drop runs inside `checksum`. -> Prints 6, 1, 1.
* a program changes: Never ends. -> Prints 7, then `1 mesh freed`.
* a program changes: Not decided (RC-5 point 1 read by type bars every back edge). -> Prints `eu-1 1`, then `1`.
* a program changes: Values queued to main wait until the C loop returns. -> They are ended at each return of `on_frame`, unless an Ember frame below keeps the point out.

**Decision (made by the owner's parameters): Where may a queued drop run while its thread holds a lock the drop takes, walks or views what it changes, waits for a scope whose tasks touch it, or is in a contract function?: A destructor sent back to the thread that made the object (a shared_ptr whose deleter runs at home) must never run inside a lock_guard it also takes, a range-for over the vector it pushes to, or while a view into what it frees is still read.**

step | who | what happens | result
1 | worker | lets go of the last owner of a connection made on main | it goes into main's queue
2 | main | holds the pool's lock and walks the free list | a loop's back edge comes up
3 | main, today | may run its queue there | the destructor waits for the lock main holds: hang
4 | main, after | runs the queue only where calling the destructor would compile, and as the lock's block ends | the id is given back

Today: The destructor may run inside the lock it needs, the loop it changes, or under a reader of what it frees.

* A: refuse such loops; you copy the list under the lock and walk the copy
* B: the compiler runs the queue only where calling the destructor would compile, and as soon as the thread leaves the place that keeps it out
* C: such objects use atomic counts everywhere (9% to 13% slower for them)

Recommendation: B: nothing to write, nothing refused, no speed lost.

Chosen: B. By: 1. Goals: A costs readability, C costs speed, B costs neither. 2. Safety: all three are safe. 3. Compile time first: A refuses correct programs, B decides when compiling and refuses none. 4. No written word under B. 5. The ordinary program is unchanged under B and longer under A. 6. Consistency: B is verdict 101's 'placed by the compiler' with verdict 57 applied where the call is placed. C undoes verdict 101's two counts.

**Decision (made by the owner's parameters): When is a value queued to an idle job worker, or to a thread between C's calls into Ember, ended?: A destructor sent back to a sleeping thread-pool worker (a deleter that must run on the thread that made the object) must still run.**

step | who | what happens | result
1 | worker job | makes a texture in a bundle for main | the worker sleeps
2 | main | clears the bundles | each texture goes to its worker's queue
3 | worker, today | sleeps; no point ever comes | never freed
4 | worker, after | is woken by its queue, frees it, sleeps again | freed at once

Today: A texture sent back to a sleeping worker is never freed while the game runs.

* A: wake the worker to run its queue; every job's end and every callback's return also run it
* B: everything a worker or a C thread makes uses atomic counts (slower for all of it)
* C: keep today: it waits for that worker's next job, maybe never

Recommendation: A: freed within one job, or at once if the worker sleeps.

Chosen: A. By: 1. Goals: B gives up the two-count speed for worker-made values. C leaves a correct program not doing what it says. 2. Safety: all three are safe. 3. Run time is needed, since no compile-time point exists here. A is the cheapest: one queue test where the thread already stops, and one wake only for a sleeping worker. A job's point comes after its end is recorded, because a worker's queue holds values from any earlier job. 4. No written word. 5. The ordinary program is unchanged. 6. A is verdict 101's 'at its next check point' for every kind of thread.

**Decision (made by the owner's parameters): A value whose maker's own check points may never run its drop: A destructor that must not be sent back to its maker: it writes main's own log with no mutex, or its maker spins for ever in a loop that promises never to wait.**

step | who | what happens | result
1 | loader job (or a helper it calls) | makes a texture whose destructor writes main's log | it goes to main in a bundle
2 | main | clears the bundles | the texture would go back to the pool thread
3 | without a fix | the pool thread runs the destructor beside main | two threads write the log: a race
4 | after | such an object is counted for every thread from the moment it is made | main runs the destructor during the clear

Today: Never freed, or, once that pool thread wakes, a race with main.

* a: count such objects for every thread from the moment they are made, wherever the making line is called from
* b: refuse the loader (main's log needs a mutex) and let the busy-poll wait for ever
* c: the pool thread passes the object on to main

Recommendation: a: nothing refused, nothing leaked, a small cost only for those objects.

Chosen: a. By: 1. Goals: b costs ease and leaves the leak. c adds a hidden path and leaves the leak. a costs atomics for those values only. 2. Safety: all three are safe, and a is safe for helpers too when judged by where the line runs. 3. b refuses a program every visible rule accepts. a reuses point 4's hand-over, and c is a second path. 4. No written word. 5. The ordinary program is unchanged. 6. a is verdict 101's fix 2 step, taken at the making line and judged per place, as point 3 judges its points.

**Decision (for the owner): A value queued to a host thread after its call into Ember has returned to C: A destructor sent back to a C library's thread that has gone back into C.**

step | who | what happens | result
1 | C loader thread | our callback makes a texture for main | the callback returns to C
2 | main | lets it go | it goes to that C thread's queue
3 | option a | freed at that thread's next callback | maybe never
4 | option b | main frees it at once, as for a finished thread | freed right away

Today: It waits until C calls into Ember on that thread again, possibly never.

* a: keep: freed at that thread's next call into Ember
* b: while a thread runs only C, whoever lets go of the last owner frees it at once; one atomic step per call from C
* c: everything made on a C thread uses atomic counts

Recommendation: b: nothing waits on a thread that may never come back.

**Decision (made by the owner's parameters): Does a thread's last check point come before or after its join is let go?: A thread's pending destructors (deleters owed to it) must have run when join() returns, as in C++.**

step | who | what happens | result
1 | worker thread | makes a connection, sends it to main | about to end
2 | main | lets it go | it goes back to the worker's queue
3 | main | joins the worker while holding the pool's lock | the worker's last step needs that lock
4 | after | refused when compiling: join after the lock's block | join always means done

Today: Not said whether a thread's last queued destructors run before or after join returns.

* A: before join returns; a join inside a lock they take is refused with a one-line fix
* B: after join returns; it compiles, but code after join may not see them done

Recommendation: A: join means done; the only refusal moves one line.

Chosen: A. By: 1. Goals: B makes `join` not mean done (ease), and results vary from run to run. 2. Safety: both are safe. 3. A decides when compiling with today's rules (THR-19 with THR-20). Its one refusal moves one line. 5. The ordinary join-then-read program is right under A and varies under B. 6. Consistency: A is verdict 43's way and THR-19's join clause as written. B narrows THR-20.

### What blocks it (1)

* E3-1 (X, contradiction): The promise of verdict 43 and THR-19, 'refused when compiling, nothing checked while running', fails for a `with scope` block inside a lock whenever a task makes a value another thread lets go of. A server that opens connections in scoped tasks and closes them on a closer thread can hang, with no message. This is a hang, not memory unsafety, so it is not S. E2-1's spawned-thread case is fine only because `thread.spawn` is on THR-20's list.
  Arbiter: Line 33 makes the Conn on the task's thread. The Conn travels inside a plain Batch, so it is not handed over (RC-4 point 4, p08 173-178; point 5, 'it travelled inside another object'). Point 4's addition does not apply either: there is no contract, and the Conn's drop reaches no module variable of main. The closer's clear puts the Conn in the task thread's queue (point 2, p08 163-168). Changed point 3 runs the last point 'before what waits for it (its `join`, its scope's end) is let go'. It also says 'for a thread other than the main one, what it may end there is counted in its function ([THR-20]), so every wait for the thread is judged with it ([THR-19], [THR-11])'. Read that way, the task changes `pool.free` under `pool.lock`, so the scope's end inside `with pool.lock:` is E7018 (THR-19, p11 272-273: 'the end of a `with scope` block, whose task changes what the held lock guards'; THR-5, p11 384-386). But the new DRP-8 sentence says these ends are 'not part of that function's effects or summary beyond what [THR-20] counts'. THR-20 counts a drop only for 'a function that may run on another thread and holds such a count' (p11 298-301). The rule's first sentence defines that phrase by a list: 'reachable from `thread.spawn`, from a job submitted outside a scope, from a callback C may run ..., or from an `@unsafe(export ...)` function' (p11 287-290). Scoped tasks are not on it. The narrow reading is forced. Read widely, THR-20's 'a module variable of the main thread they reach is `E7012`' would clash with the same rule's 'reached only by the main thread and the tasks of scopes it opens' (p11 285-286). It would also make the change's own tests scoped_job_value_freed_on_main and scoped_job_helper_value_freed_on_main E7012 instead of `1002`, because their jobs hold an away count of the Texture from the making line. So THR-20 counts nothing for this task, and by DRP-8 the scope's end compiles. Two rules give two outcomes for line 40: E7018, or compile and hang. Without the change, the order of the last point against the scope's end is not stated. The definite hang, and both sentences in conflict, come from the change.

```
import std.thread
from std.sync import Atomic, Lock

static DONE: Atomic[int] = Atomic(0)

class Pool:                                   # a connection pool: a free list behind a lock (std::mutex + std::vector)
    lock: Lock = Lock()
    free: Array[int] guarded by lock = []

class Conn:                                   # one connection, made on a scoped task's thread
    id: int
    pool: thread_shared Pool
    fn drop():                                # gives its id back to the pool
        with this.pool.lock:
            this.pool.free.push(owns copy of this.id)

class Batch:                                  # a plain object: connections to close
    conns: Array[thread_shared Conn] = []

class Inbox:                                  # batches waiting for the closer thread
    lock: Lock = Lock()
    items: Array[Batch] guarded by lock = []

fn close_all(inbox: thread_shared Inbox):     # the closer thread: lets the batches go
    while DONE.load() == 0:
        with inbox.lock:
            inbox.items.clear()               # ends each Batch and its Conn: the Conn goes back to the queue of the thread that made it
    with inbox.lock:
        inbox.items.clear()

fn open_one(pool: thread_shared Pool, inbox: thread_shared Inbox):
    with inbox.lock:
        inbox.items.push(owns Batch(conns=[thread_shared Conn(id=1, pool=thread_shared pool)]))

fn main():
    pool = thread_shared Pool()
    inbox = thread_shared Inbox()
    closer = thread.spawn(owns fn() thread_shared inbox: close_all(inbox))
    with pool.lock:
        with s = thread.scope():
            s.spawn(owns fn(): open_one(pool, inbox))   # danger in C++ terms: a scope's join waiting on a thread whose pending destructor needs the mutex this thread holds
        println(pool.free.len())
    DONE.store(1)
    closer.join()
    with pool.lock:
        println(pool.free.len())
```

Arbiter's rulings on the open objections: 3 real, 2 refuted, 0 already in today's specification

Objections closed in the debate: 14 (14 fixed, 0 withdrawn after a rebuttal)

Gaps found on the way that exist without the change: 2

