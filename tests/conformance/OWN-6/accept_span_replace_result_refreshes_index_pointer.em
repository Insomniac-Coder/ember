#$ test: run-pass
#$ rules: OWN-6, SPN-1, SPN-2, CG-C-3
#$ profiles: debug, release, shipping
#$ stdout: 10 20 3
#$ stdout: 20 30 2
# The old Span returned directly by mem.replace is
# dynamically indexed afterward. Its descriptor assignment must also refresh
# any typed duplicate pointer selected for this direct result local. A runtime
# index matters: ConstIndex rendering does not use the duplicate pointer.

@noinline
fn replace_shared_view(index: int):
    values = [10, 20, 30]
    view: Span[int] = values
    replacement = values[1..3]
    old = mem.replace(view, replacement)
    println(old[index], old[index + 1], old.len())
    println(view[index], view[index + 1], view.len())

fn main():
    replace_shared_view(0)
