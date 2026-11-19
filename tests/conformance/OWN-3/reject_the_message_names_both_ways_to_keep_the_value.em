#$ test: compile-fail
#$ rules: OWN-3

fn main():
    items = [1, 2]
    snapshot = items
    println(items.len(), snapshot.len())    #$ error[E3040]: `items` was handed over to `snapshot` on line 6 and is used on line 7; to keep both, write `snapshot = copy of items` (a copy of your own) or `snapshot = shares items` (two owners of one list)
