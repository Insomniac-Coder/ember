#$ test: compile-fail
#$ rules: BRW-18, CTL-2
#$ help: note after the loop: move line 20 below the loop
## design rules: R5, R8
## R5, R8 (c), row 3: what a method changes is worked out from its body (verdict 86): `note` pushes to
## `this.log`, so `this.note(...)` inside a loop over `this.log` is refused (E3020), and the message names
## the chain; inside a loop over `this.players` it is allowed
## (`accept_tick_calls_note_inside_a_loop_over_players`). The nearest program that is allowed is
## `accept_a_method_that_adds_to_the_log_called_after_the_loop_over_it`.

class Game:
    log: Array[String]
    frame: int = 0

    fn note(msg: String):
        this.log.push(f"[{this.frame}] {msg}")

    fn replay():
        for line in this.log:
            println(line)
            this.note(String("replayed"))    #$ error[E3020]: `this.note(...)` (line 20) adds to `this.log` (through `note`, line 15) while this loop walks `this.log` (line 18)

fn main():
    g = Game(log=[String("start")])
    g.replay()
