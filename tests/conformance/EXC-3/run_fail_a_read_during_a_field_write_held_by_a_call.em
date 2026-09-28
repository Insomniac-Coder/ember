#$ test: run-fail
#$ rules: EXC-3, EXC-2, EXC-19
#$ profiles: debug, release, shipping
#$ panics: exclusivity violation: read access to Player.items while a write access to Player.items is active
# `[EXC-3]` — a check is removed only when nothing held anywhere could make it
# fail. Passing `p.items` to a `mut` parameter holds a write to it for the
# call, so the read of `other.items` inside keeps its check and panics.

class Player:
    items: Array[int]

    fn init(mut self):
        self.items = [1]

fn grow(mut items: Array[int], other: Player):
    items.push(len(other.items))

fn main():
    p = Player()
    alias = p
    grow(p.items, alias)
