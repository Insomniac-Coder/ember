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

    fn apply(e: Edit):
        match e:
            Insert(pos, text):
                this.text = f"{this.text[..pos]}{text}{this.text[pos..]}"
            Delete(pos, text):
                this.text = f"{this.text[..pos]}{this.text[pos + text.len()..]}"

    fn insert(pos: int, text: StrView):
        e = Edit.Insert(pos, text.to_string())
        this.apply(e)
        this.undo_stack.push(owns e)
        this.redo_stack.clear()

    fn delete(pos: int, count: int):
        removed = this.text[pos..pos + count]
        e = Edit.Delete(pos, removed.to_string())
        this.apply(e)
        this.undo_stack.push(owns e)
        this.redo_stack.clear()

    fn undo() -> bool:
        match this.undo_stack.pop():
            Some(e):
                this.apply(inverse(e))
                this.redo_stack.push(owns e)
                return true
            None:
                return false

    fn redo() -> bool:
        if this.redo_stack.pop() exists as e:
            this.apply(e)
            this.undo_stack.push(owns e)
            return true
        return false

    fn show(label: StrView):
        println(f"{label:<10} |{this.text}|  undo={this.undo_stack.len()} redo={this.redo_stack.len()}")

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
