# K01-sharedref-send

Question: May a struct holding a sharedref mut link be sent to another thread?
Severity: high; impact rank 1
Members: J2-06; primary J2-06
Why together: Single question.
Critic's notes: Data race in safe code if left open. A keeps verdict 77's two-link reduction, because scoped tasks reach the link through the caller's variables (THR-11), which needs no Send. Same one-thread-header principle as K03 (J7-04).

Status: answered by the owner, verdict 164 (tasks/96/redesign/JOB.md).

## Finding J2-06
Rules: TYP-36, marker table (Send, ThreadShared), THR-8, THR-9, THR-13; file tasks/spec-0.9.9/parts/p04-types.md lines 577-578; kind undecided_case; severity high
Quote from Hardened_11:
`ref T` and `Span[T]` are when `T` is `ThreadShared`, `ref mut T` and `MutSpan[T]` when `T` is `Send`

The finder's question:
Is a `sharedref mut T` link, or a struct holding one, `Send`? Is it `ThreadShared`? Does `[T: needs Send]`
accept a `Watcher`, and does `thread_shared` accept one inside it?

Program:
```
class Player:
    hp: int = 10

struct Watcher:
    target: sharedref mut Player

[T: needs Send]
fn hand_off(owns item: T, tx: Sender[T]):
    tx.send(owns item).expect("gone")

fn main():
    p = Player()
    w = Watcher(target=sharedref mut p)
    tx, rx = channel[Watcher](capacity=1)
    hand_off(owns w, tx)       # Send by THR-8's list, or refused?
    p.hp -= 1                  # the other thread may write p.hp through its holder
```

The checker's verdict: confirmed (route owner_odr, severity high, confidence 0.65)
Reason:
THR-8 (p11 43-50) and the marker table (p04 577) list what is not Send (shares, shares mut, a weakref to one,
thread-bound values) and which views are Send (`ref T`/`Span` when T is ThreadShared, `ref mut T`/`MutSpan`
when T is Send), never `sharedref mut T`; THR-9 (p11 64-65) and the table (p04 578) leave it out of
ThreadShared's list too. Inferred by components, a struct holding one would count as Send and ThreadShared.
THR-13 (p11 149-151) only says what scoped tasks may do with a `sharedref mut` target and that an unscoped
thread never holds one; verdict 3 says 'on two threads, only with a lock', and verdict 77 lets scoped tasks
reduce through two links. Those cover tasks that use the caller's variables (THR-11 p11 388-389, no Send
needed), not a link handed through a channel. The finder's fix (not Send) keeps verdict 77, since scoped tasks
reach the link without Send.

options:
[{'letter': 'A', 'summary': "A `sharedref mut` link is neither Send nor ThreadShared, as a `shares mut` owner
is: it is never handed or sent to another thread. Scoped tasks still reach it by using the caller's variables,
where the compiler sees what each task changes (two tasks doing `a.target.hp -= 1` and `b.target.hp -= 2`
still add up).", 'code': 'tx, rx = channel[Watcher](capacity=1)   # refused', 'outcome': 'Refused (E7015: a
`Watcher` holds `target`, a changeable link other names share on this thread; let the task use `w` where it is
instead of sending it). No race is possible.'}, {'letter': 'B', 'summary': 'A `sharedref mut T` link is Send
when T is, as `ref mut T` is, and goes only to scoped tasks; never ThreadShared.', 'code': 'tx, rx =
channel[Watcher](capacity=1)   # accepted', 'outcome': 'Compiles as written unless the compiler follows the
link through the channel to see that the task changes `p.hp` while `main` does, which no rule says; without
that, the task and `main` write `p.hp` at the same time (a data race).'}]

recommendation:
{'letter': 'A', 'reason': '`ref mut` may cross to a task because it is the only name for its place; a
`sharedref mut` link never is, so it belongs with `shares mut`. A refuses a race when compiling and loses
nothing the reductions need.', 'checklist': "2 (refuses only what is wrong), 3 (decided when compiling), 6
(the nearest rule, `shares mut`'s), 8 (a forgotten lock is a refusal, not a race)"}

