#$ test: compile-fail
#$ rules: OWN-3
#$ help: to keep both, write `kept = copy of items` on line 13
## design rules: R1
## R1: a name that may have been handed over on some path is ended for every later use: the
## compiler does not decide at run time which branch ran (today's `[OWN-4]` "a move missing on some
## path").

fn main():
    items = [1, 2, 3]
    keep = true
    kept: Array[int] = []
    if keep:
        kept = items
    println(items.len())       #$ error[E3040]: `items` may have been handed over to `kept` on line 13 (when `keep` is true) and is used on line 14
