#$ test: compile-fail
#$ rules: MOD-3
#$ help: write `fn count(log: thread_shared Array[String]) -> int:`
## design rules: R3, R15
## R3 and the removed words of `V4/rules.md` section 3: `SyncShared[T]` is written `thread_shared T` (E1010; the
## `SyncShared` row of `V4/vocabulary.md`; completeness gap G8). The nearest program that is allowed is
## `accept_thread_shared_on_a_value_made_thread_shared`.

fn count(log: SyncShared[Array[String]]) -> int:      #$ error[E1010]: `SyncShared` is not part of Ember
    return log.len()

fn main():
    println(0)
