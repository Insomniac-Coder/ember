# A tiny text editor buffer whose insert and delete edits can be undone and redone with two stacks.

enum Edit:
    Insert(pos: int, text: String)
    Delete(pos: int, text: String)

fn inverse(e: Edit) -> Edit:
    return match e:
        Insert(pos, text) => Edit.Delete(pos, text.clone())
        Delete(pos, text) => Edit.Insert(pos, text.clone())

class Editor:
    text: String = ""
    undo_stack: Array[Edit] = []
    redo_stack: Array[Edit] = []

    fn apply(self, e: Edit):
        match e:
            Insert(pos, text):
                self.text = f"{self.text[..pos]}{text}{self.text[pos..]}"
            Delete(pos, text):
                self.text = f"{self.text[..pos]}{self.text[pos + text.len()..]}"

    fn insert(self, pos: int, text: str):
        e = Edit.Insert(pos, text.to_string())
        self.apply(e)
        self.undo_stack.push(e)
        self.redo_stack.clear()

    fn delete(self, pos: int, count: int):
        removed = self.text[pos..pos + count]
        e = Edit.Delete(pos, removed.to_string())
        self.apply(e)
        self.undo_stack.push(e)
        self.redo_stack.clear()

    fn undo(self) -> bool:
        match self.undo_stack.pop():
            Some(e):
                self.apply(inverse(e))
                self.redo_stack.push(e)
                return true
            None:
                return false

    fn redo(self) -> bool:
        if Some(e) = self.redo_stack.pop():
            self.apply(e)
            self.undo_stack.push(e)
            return true
        return false

    fn show(self, label: str):
        println(f"{label:<10} |{self.text}|  undo={self.undo_stack.len()} redo={self.redo_stack.len()}")

fn main():
    ed = Editor()
    ed.insert(0, "Hello world")
    ed.show("insert")
    ed.insert(5, ",")
    ed.show("comma")
    ed.insert(ed.text.len(), "!")
    ed.show("bang")
    ed.delete(0, 5)
    ed.show("delete")
    ed.insert(0, "Goodbye")
    ed.show("insert")

    while ed.undo():
        ed.show("undo")
    ed.redo()
    ed.redo()
    ed.show("redo x2")

    ed.insert(0, ">> ")
    ed.show("new edit")
    if not ed.redo():
        println("nothing to redo after a new edit")
