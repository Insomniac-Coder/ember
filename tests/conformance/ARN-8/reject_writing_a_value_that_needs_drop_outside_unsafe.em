#$ test: compile-fail
#$ rules: ARN-8, UNS-1
# `[ARN-8]` — a `MaybeUninit` never runs its value's clean-up, so for a `T` that needs drop,
# `uninit()`, `write` and `write_at` are `unsafe` operations, `E3100` outside `unsafe:`; for a `T`
# with nothing to drop they stay safe. `write(owns v: T)` says `owns` at the call (`[FN-2a]`).

struct Player:
    name: String

fn main():
    slot: MaybeUninit[Player] = MaybeUninit[Player].uninit()  #$ error[E3100]: `MaybeUninit` never runs a `Player`'s clean-up, so a value written into it may never be freed; write it inside `unsafe:`, or keep the value in an `Option[shares Player]`
    slot.write(owns Player(String("ann")))  #$ error[E3100]: `MaybeUninit` never runs a `Player`'s clean-up, so a value written into it may never be freed; write it inside `unsafe:`, or keep the value in an `Option[shares Player]`
    count: MaybeUninit[int] = MaybeUninit[int].uninit()
    count.write(owns 3)
    count.write(4)  #$ error[E3028]: `write` takes `4` over
    println(1)
