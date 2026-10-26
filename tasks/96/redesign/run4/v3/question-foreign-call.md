# Open question: does a function that calls C say what C's kept callback changes?

## The program (C changes `Game` and the global `LOG`)

```ember
static mut LOG: Array[String] = []

class Game:
    score: i32 = 0

    fn add_point() changes score, LOG:
        this.score += 1
        LOG.push(String("point"))

unsafe extern "C":
    fn unregister(cb: extern "C" fn(*mut void), ud: *mut void)
    @ffi(param(cb, callback(retained, user_data=ud), threads(creator)), destroys(unregister))
    safe fn register_cb(cb: extern "C" fn(*mut void), ud: *mut void)
    safe fn poll_events()

fn setup(owns game: shares mut Game) changes Game, LOG -> Retained[ForeignCallback]:
    return register_cb(fn() with owns game changes Game, LOG: game.add_point())

fn tick():
    poll_events()

fn main() changes Game, LOG:
    game = shares mut Game()
    token = setup(shares mut game)
    tick()
    println(game.score)
```

`setup` hands C a function, and C keeps it. `main` calls `tick()`, `tick` calls the C function
`poll_events()`, and during that call C runs the kept function: the game gets a point and `LOG` gets a
line. So calling `tick()` changes `Game` and `LOG`, and `fn tick():` says nothing about it.

## The question

- **(a)** `tick`'s line says it: `fn tick() changes Game, LOG:`
- **(b)** it does not: `fn tick():`

## Only if (a): the three cases

**Case 1, in your own program:** `fn tick() changes Game, LOG:`

**Case 2, with your named list (strict):**

```ember
world: changelist = (Game, LOG)

fn tick() changes world:
    poll_events()
```

**Case 3, the same `tick` in a library**, a separate file that cannot name your `Game` or your `LOG`.
Two things change, so `captured...`:

```ember
pub fn wait_frame() changes captured...:
    poll_events()
```

(If the kept function changed only one thing, say only `LOG`, it would be `changes captured`.)

## Status

Not answered. Nothing about it is in the rules yet.
