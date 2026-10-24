/* A25 memory attack on the first attack's saved take-along rule (phase3b-containers-attack.md 6.3 and
   section 9, A25 rule 2): "views of a text that dies stored in a list that lives longer move that text
   into the list's owner block ... clear() frees the block".
       words = []                                    words: Array[str] = []
       for line in text.split("\n"):                 for line in text.split("\n"):
           words.extend(line.lower().split())            low = line.to_lower()
                                                         for w in low.split_whitespace(): words.push(w)
       best = words[0]                               best = words[0]
       words.clear()                                 words.clear()
       print(best)                                   println(best)
   `best` is a name for words[0] (A2 rule 1: a str is not a plain number). clear() removes the named
   element, so A2 rule 3 moves its value, a view, into `best`; but the same clear() frees the owner
   block that holds the text the view points into. Python prints "alpha" (probes/py_attack2_output.txt).
   SAVED: a view read out of a take-along list lives no longer than the block it points into; when
   such a view is still used after clear(), the compiler moves the block's free from clear() to that
   view's last use (the keep-alive the answers report already uses for a reassigned text), and when
   the view is used across a loop that clears again (the blocks would pile up without bound), it is
   a compile-time error with the fix-it `best = words[0].to_string()`. Decided when compiling.
   Poisoning allocator: a freed block is filled with 0xDD and kept; nothing crashes. */
#include "../../../redesign/bench/harness.h"
#include <string.h>
#include <ctype.h>

typedef struct { uint64_t magic, size; } Hdr;
#define LIVE 0x4C4956454C495645ull
static void *xmalloc(size_t n) { Hdr *h = malloc(sizeof(Hdr) + n); h->magic = LIVE; h->size = n; return h + 1; }
static void xfree(void *p) { if (!p) return; Hdr *h = (Hdr *)p - 1; h->magic = 0; memset(p, 0xDD, h->size); }

typedef struct { const char *p; size_t n; } View;
typedef struct { char *texts[16]; size_t n; } Owner;                  /* the list's owner block */
typedef struct { View *data; size_t len, cap; Owner *owner; } Words;   /* the 8-byte owner pointer */

static void take_line(Words *ws, const char *line, size_t n) {         /* lowered line, taken along */
    char *low = xmalloc(n + 1);
    for (size_t k = 0; k < n; k++) low[k] = (char)tolower((unsigned char)line[k]);
    low[n] = 0;
    if (!ws->owner) { ws->owner = xmalloc(sizeof(Owner)); ws->owner->n = 0; }
    ws->owner->texts[ws->owner->n++] = low;
    for (size_t a = 0; a < n;) {
        while (a < n && low[a] == ' ') a++;
        size_t b = a; while (b < n && low[b] != ' ') b++;
        if (b > a) ws->data[ws->len++] = (View){low + a, b - a};
        a = b;
    }
}
static void clear_words(Words *ws, int keep_block) {
    ws->len = 0;
    if (keep_block) return;                       /* SAVED: freed at best's last use instead */
    for (size_t k = 0; k < ws->owner->n; k++) xfree(ws->owner->texts[k]);
    ws->owner->n = 0;
}
static void drop_block(Words *ws) { for (size_t k = 0; k < ws->owner->n; k++) xfree(ws->owner->texts[k]); ws->owner->n = 0; }

static void run(int saved) {
    const char *text = "Alpha beta\nGamma delta";
    Words ws = {xmalloc(16 * sizeof(View)), 0, 16, NULL};
    for (const char *s = text; *s;) {
        const char *e = strchr(s, '\n'); size_t n = e ? (size_t)(e - s) : strlen(s);
        take_line(&ws, s, n);
        s += n + (e ? 1 : 0);
    }
    View best = ws.data[0];                       /* best = words[0]; rule 3 keeps its value on removal */
    const char *line0 = ws.owner->texts[0];       /* the taken text best points into */
    clear_words(&ws, saved);                      /* words.clear() */
    int freed = ((const Hdr *)line0 - 1)->magic != LIVE;
    printf("  %-8s print(best) reads %s: %.*s\n", saved ? "saved:" : "written:",
           freed ? "FREED memory" : "live memory", freed ? 7 : (int)best.n, freed ? "<freed>" : best.p);
    if (saved) drop_block(&ws);                   /* best's last use was the line above */
}
int main(void) {
    h_start();
    printf("best = words[0]; words.clear(); print(best)   (Python prints alpha)\n");
    run(0); run(1);
    return 0;
}
