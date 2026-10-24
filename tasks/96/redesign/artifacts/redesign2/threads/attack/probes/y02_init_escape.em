# PROPOSED syntax. "Fields never written after init need no lock" (4.5) and "no plain field written after
# the share" (4.6) leave out today's [THR-1] clause about init (x04_sync_init_escape.em: E7003).
import std.thread

class Worker:
    name: String = "".to_string()

    fn init(self, name: String):
        thread.spawn(fn(): println(self.name))   # self is shared here
        self.name = name                         # a write in init, racing the thread's read of a String
