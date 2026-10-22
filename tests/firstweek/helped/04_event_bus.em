# An event bus: subscribers register closures per topic, publishers post game events, and the bus delivers them.

enum Event:
    PlayerJoined(name: String)
    PlayerLeft(name: String)
    ScoreChanged(name: String, score: int)
    Chat(name: String, text: String)

fn topic_of(e: Event) -> str:
    return match e:
        PlayerJoined(_) => "players"
        PlayerLeft(_) => "players"
        ScoreChanged(_, _) => "score"
        Chat(_, _) => "chat"

class EventBus:
    subscribers: Map[String, Array[fn(Event)]] = {}
    delivered: int = 0

    fn subscribe(self, topic: str, owned handler: fn(Event)):
        if topic not in self.subscribers:
            self.subscribers[topic] = []
        self.subscribers[topic].push(owned fn(e) => handler(e))

    fn publish(self, event: Event):
        topic = topic_of(event)
        if topic not in self.subscribers:
            println(f"(nobody listens to {topic})")
            return
        for handler in self.subscribers.get_mut(topic).unwrap().iter_mut():
            handler(event)
            self.delivered += 1

class Scoreboard:
    scores: Map[String, int] = {}

    fn on_event(self, e: Event):
        match e:
            PlayerJoined(name): self.scores[name] = 0
            PlayerLeft(name): self.scores.remove(name)
            ScoreChanged(name, score): self.scores[name] = score
            _: pass

    fn show(self):
        for name, score in self.scores.items():
            println(f"  {name}: {score}")

class ChatLog:
    lines: Array[String] = []

    fn on_event(self, e: Event):
        if Chat(name, text) = e:
            self.lines.push(f"<{name}> {text}")

fn main():
    bus = EventBus()
    board = Scoreboard()
    log = ChatLog()
    bus.subscribe("players", owned fn(e) => board.on_event(e))
    bus.subscribe("score", owned fn(e) => board.on_event(e))
    bus.subscribe("chat", owned fn(e) => log.on_event(e))
    bus.subscribe("players", owned fn(e) => println(f"[audit] {e!r}"))

    bus.publish(Event.PlayerJoined("ann"))
    bus.publish(Event.PlayerJoined("bob"))
    bus.publish(Event.Chat("ann", "hello!"))
    bus.publish(Event.ScoreChanged("bob", 15))
    bus.publish(Event.ScoreChanged("ann", 9))
    bus.publish(Event.Chat("bob", "gg"))
    bus.publish(Event.PlayerLeft("ann"))

    println("scores:")
    board.show()
    println("chat:")
    for line in log.lines:
        println(f"  {line}")
    println(f"{bus.delivered} deliveries")
