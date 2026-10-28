#$ test: compile-fail
#$ rules: MOD-3
#$ help: write `items = shares [1, 2]` for owners that read the list, or `items = shares mut [1, 2]` for owners that may also change it
## design rules: R3
## R3 and the removed words of `V4/rules.md` section 3: `Shared[T]` is written `shares T` or `shares mut T` (E1010;
## the `Shared` row of `V4/vocabulary.md`; completeness gap G8). The nearest program that is allowed is
## `accept_two_owners_that_may_change_one_list`.

fn main():
    items: Shared[Array[int]] = [1, 2]     #$ error[E1010]: `Shared` is not part of Ember
    println(items.len())
