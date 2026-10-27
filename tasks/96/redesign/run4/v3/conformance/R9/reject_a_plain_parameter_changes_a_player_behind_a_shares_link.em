#$ test: compile-fail
#$ rules: R9, R5, R4
## R9, R5: "a plain name never changes anything", an object behind a `shares` link included; the
## players the watchers share are changed through a name that may change the list (AN-51).

class Player:
    hp: int = 1000

struct Watcher:
    target: shares mut Player

fn hurt(ws: Array[Watcher]):
    for i in 0..ws.len():
        ws[i].target.hp -= 1      #$ error[E3023]: `ws[i].target.hp -= 1` changes the player `ws[i].target` shares, but `ws` only reads the caller's watchers; write `fn hurt(ref mut ws: Array[Watcher]):` and call it `hurt(ref mut ws)`

fn main():
    p = Player()
    ws = [Watcher(target=shares mut p)]
    hurt(ws)
