/* retarget_body.h: run 3's re-pointed shared targets workload (AN-52 (a)), unchanged:
   target/redesign3/10-area-C/bench/share_c.c (same work, seed 42, checksum 80363660820).
   262,144 players from one slab block, 1,048,576 watchers, 32 frames; each step: t.hp -= 1;
   acc += t.x * (len(t.inventory) + 1); one step in eight pushes to the inventory (and past 8 sums and
   clears it); one step in sixteen re-points the watcher at a random player:
       ws[i].target = shares mut players[r]      # +1 on the new player, -1 on the old one
   The view `t` of ws[i].target is not counted: nothing in its life ends a Player ([RC-2b], [RC-5]
   point 1, R10). Integer overflow checks are left out everywhere (not a memory-model cost), as run 3.
   Include after counts.h. */
#define P_COUNT (1u << 18)
#define W_COUNT (1u << 20)
#define FRAMES 32

typedef struct { Player *target; } Watcher;

static Player *slab; static uint32_t slab_used;
static Player *new_player(uint32_t i) {
    if (!slab) { slab = (Player *)malloc((size_t)P_COUNT * sizeof(Player)); if (!slab) abort(); }
    Player *p = &slab[slab_used++];
    memset(p, 0, sizeof *p);
    player_init_header(p);
    p->hp = 1000; p->x = (int32_t)(i & 1023);
    return p;
}

int main(void) {
    h_start();
    h_seed(42);
    Player **players = (Player **)malloc(P_COUNT * sizeof(Player *));
    Watcher *ws = (Watcher *)malloc(W_COUNT * sizeof(Watcher));
    if (!players || !ws) abort();
    for (uint32_t i = 0; i < P_COUNT; i++) players[i] = new_player(i);
    static volatile size_t vlen = P_COUNT;
    size_t players_len = vlen; (void)players_len;            /* the list's length, not known when compiling */
    for (uint32_t i = 0; i < W_COUNT; i++) {
        uint32_t r = h_rand() & (P_COUNT - 1);
        Player *t = players[r];
        retain(t);                                           /* ws[i] = Watcher(target=shares mut players[r]) */
        ws[i].target = t;
    }
    uint64_t acc = 0;
    for (uint32_t f = 0; f < FRAMES; f++) {
        for (uint32_t i = 0; i < W_COUNT; i++) {
            Player *t = ws[i].target;                        /* t = ref mut ws[i].target: no count */
            t->hp -= 1;
            acc += (uint64_t)(uint32_t)t->x * (uint64_t)(t->inv.len + 1);
            if ((i & 7) == (f & 7)) {
                ivec_push(&t->inv, (int32_t)f);
                if (t->inv.len > 8) {
                    for (size_t k = 0; k < t->inv.len; k++) acc += (uint64_t)(uint32_t)t->inv.p[k];
                    t->inv.len = 0;
                }
            }
            if ((h_rand() & 15) == 0) {
                uint32_t r = h_rand() & (P_COUNT - 1);
#if VARIANT != V_PLAIN
                if (UNLIKELY(r >= players_len)) e_stop("index out of bounds");   /* bounds check, as today */
#endif
                Player *n = players[r];
                retain(n);                                   /* the new owner first ... */
                Player *old = ws[i].target;
                ws[i].target = n;
                release(old);                                /* ... then the old one ends */
            }
        }
    }
    uint64_t sum = acc;
    for (uint32_t i = 0; i < P_COUNT; i++) sum += (uint64_t)(uint32_t)players[i]->hp + players[i]->inv.len;
    h_report(NAME, sum);
    return 0;
}
