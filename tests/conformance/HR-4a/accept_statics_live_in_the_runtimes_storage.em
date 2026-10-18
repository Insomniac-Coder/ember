#$ test: run-pass
#$ rules: HR-4a, HR-17
#$ stdout: 3 a=1
#$ assert-c: contains("ember_rt_static_begin(em_rl_static_state[")
#$ assert-c: contains("em_rl_static_data[")
#$ assert-c: !contains("static uint32_t state;")
# In a reloadable build (this directory's manifest says `reload = "all"`) a static's storage and
# its first-access state are the runtime's, found by the static's name, never the image's: a reload
# loads a new image and the static keeps its value.

from std.sync import Atomic, Mutex

static HITS: Atomic[int] = Atomic(0)
static NAMES: Mutex[Map[String, int]] = Mutex({})

fn hit(name: str):
    HITS.fetch_add(1)
    with names = NAMES.lock():
        names[name.to_string()] = HITS.load()

fn main():
    hit("a")
    hit("b")
    hit("c")
    with names = NAMES.lock():
        first = names["a"]
        println(HITS.load(), f"a={first}")
