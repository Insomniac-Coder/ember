# A14: the Python program, async with a task group (async is reserved in Ember 0.9.10).
async fn fetch() -> int:
    return 1

async fn serve():
    cache = [1, 2, 3]
    n = await fetch()
    println(cache.len() + n)
