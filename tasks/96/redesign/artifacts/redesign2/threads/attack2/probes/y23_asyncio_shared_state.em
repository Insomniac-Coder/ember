# Proposed syntax (async is reserved for v2; design only). Python's asyncio habit: tasks of one event loop
# change shared state between awaits with no lock, because only one task runs at a time. Under the scope
# rule applied to task groups (answers report 7.3 rule 3: "tasks that may run at the same time are checked
# against each other, on one thread or many"), two tasks that add to `seen` and read its length conflict.
async fn crawl(url: String, seen: Set[String], out: Array[String]):
    page = await fetch(url)
    if url not in seen:
        seen.add(url)
        if len(seen) <= 100:
            out.push(page.title)

async fn main():
    seen: Set[String] = {}
    out: Array[String] = []
    async with tasks = TaskGroup():
        for u in URLS:
            tasks.spawn(crawl(u, seen, out))
