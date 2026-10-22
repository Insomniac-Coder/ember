# A to-do list: add tasks with priorities and tags, complete them, filter by status and tag, and print a summary.

enum Priority:
    Low
    Medium
    High

enum Status:
    Open
    Done

struct Task:
    id: int
    title: String
    priority: Priority
    status: Status = Status.Open
    tags: Array[String] = []

fn priority_label(p: Priority) -> str:
    return match p:
        Low => "low"
        Medium => "medium"
        High => "HIGH"

fn priority_rank(p: Priority) -> int:
    return match p:
        High => 0
        Medium => 1
        Low => 2

class TodoList:
    tasks: Array[Task] = []
    next_id: int = 1

    fn add(self, title: str, priority: Priority, tags: Array[String] = []) -> int:
        id = self.next_id
        self.next_id += 1
        self.tasks.push(Task(id=id, title=title.to_string(), priority=priority, tags=tags.clone()))
        return id

    fn complete(self, id: int) -> bool:
        for task in self.tasks.iter_mut():
            if task.id == id:
                task.status = Status.Done
                return true
        return false

    fn open_tasks(self) -> Array[Task]:
        return [t.clone() for t in self.tasks if t.status == Status.Open]

    fn with_tag(self, tag: str) -> Array[Task]:
        return [t.clone() for t in self.tasks if tag.to_string() in t.tags]

    fn print(self):
        sorted_tasks = self.tasks.clone()
        sorted_tasks.sort_by_key(fn(t) => priority_rank(t.priority))
        for t in sorted_tasks:
            mark = "x" if t.status == Status.Done else " "
            println(f"[{mark}] #{t.id} {t.title} ({priority_label(t.priority)})")

fn main():
    todo = TodoList()
    milk = todo.add("buy milk", Priority.Low, ["shopping"])
    report = todo.add("finish report", Priority.High, ["work"])
    todo.add("call plumber", Priority.Medium, ["home"])
    todo.add("book flights", Priority.High, ["travel", "home"])
    todo.add("water plants", Priority.Low)
    todo.print()
    println("")

    todo.complete(report)
    todo.complete(milk)
    if not todo.complete(99):
        println("there is no task #99")
    todo.print()
    println("")

    open = todo.open_tasks()
    println(f"{open.len()} open tasks")
    for t in todo.with_tag("home"):
        println(f"home: {t.title}")
    done = todo.tasks.len() - open.len()
    percent = done * 100 // todo.tasks.len()
    println(f"{percent}% done")
