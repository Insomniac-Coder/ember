# A safe wrapper around a thread-bound C object (a window that its library allows only on the thread
# that made it) moved to another thread. Today: refused, because a raw pointer is not Send ([THR-8]).
# The proposal removes Send; nothing else in it says this value may not cross.
import std.thread

unsafe extern "C":
    fn win_open() -> *mut u8
    fn win_swap(w: *mut u8)

struct Window:
    raw: *mut u8

    fn swap(self):
        # SAFETY: raw came from win_open and this Window is used on the thread that opened it
        unsafe:
            win_swap(self.raw)

fn open() -> Window:
    # SAFETY: win_open has no preconditions
    unsafe:
        return Window(raw=win_open())

fn main():
    w = open()
    h = thread.spawn(owned fn(): w.swap())
    h.join()
