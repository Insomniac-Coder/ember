#$ test: compile-pass
#$ rules: R9, R10, R5
## R9: "Any `ref mut` name of it ... may change it"; R10 point 1: nothing in `t`'s life can end an
## object of type `Player`, so `t` costs no count (AN-51: 0 count operations per step where today makes
## 2). The change reaches the player through a link, so `hurt` says `changes Player` although `ws` came
## in as `ref mut` (R5, verdict 63), and so does `main`, which calls it.

class Player:
    hp: int = 1000

struct Watcher:
    target: shares mut Player

fn hurt(ref mut ws: Array[Watcher]) changes Player:
    for i in 0..ws.len():
        t = ref mut ws[i].target      # a name for the watcher's link to its player: no count
        t.hp -= 1

fn main() changes Player:
    p = Player()
    ws = [Watcher(target=shares mut p)]
    hurt(ref mut ws)
    println(p.hp)
