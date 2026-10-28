#$ test: compile-fail
#$ rules: BRW-18, CTL-2
#$ help: move line 29, `cam.empty_target()`, below the loop
## design rules: R7, R8, R5
## R7, R8 (c): a list in a `sharedref mut` target may not change while a `for` loop walks it, through a
## call that may change it either: the compiler works out from `empty_target`'s body that it clears the
## inventory of the player `cam.target` links to (R5, verdict 86), and `cam.target` may be the player
## whose inventory this loop walks. AN-48, point 6. A method that writes only a number of the player may
## run in such a loop: `accept_a_method_that_changes_a_number_of_the_target_inside_a_loop_over_its_list.em`.

struct Player:
    hp: int
    inventory: Array[String]

struct Camera:
    target: sharedref mut Player

    fn empty_target():
        this.target.inventory.clear()

struct Hud:
    target: sharedref mut Player

fn main():
    p = Player(hp=100, inventory=[String("key")])
    cam = Camera(target=sharedref mut p)
    hud = Hud(target=sharedref mut p)
    for item in hud.target.inventory:
        println(item)
        cam.empty_target()    #$ error[E3020]: `cam.empty_target()` (line 29) clears `cam.target.inventory` (through `empty_target`, line 18), and `cam.target` may be the `Player` whose inventory this loop walks (`hud.target.inventory`, line 27); call it after the loop: move line 29, `cam.empty_target()`, below the loop
