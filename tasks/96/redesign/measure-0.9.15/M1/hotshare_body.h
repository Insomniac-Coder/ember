/* hotshare_body.h: the owner count's worst case. Every step makes one `shares mut` owner and ends one,
   on players that stay in the level-1/level-2 cache, so the count work is most of the step and the
   limit compare is not hidden behind a cache miss (in retarget_body.h it is: one re-point in sixteen
   steps, on a random player out of 262,144).
       for f in 0..FRAMES:
           for i in 0..len(ws):
               t = players[(3 * i + f) & 1023]          # a view: no count ([RC-2b])
               ws[i].target = shares mut t              # +1 on t ([RC-1]), -1 on the old target
               acc += t.x
   1,024 players (x = their position), 4,096 watchers, 20,000 frames: 81,920,000 re-points.
   Checksum, worked out without running: 3 is odd, so (3i + f) & 1023 takes every value 0..1023 four
   times as i runs over 0..4095, and each frame adds 4 * (0 + ... + 1023) = 2,095,104; the final pass
   over the watchers adds the same once more: (20,000 + 1) * 2,095,104 = 41,904,175,104.
   Include after counts.h. */
#define HP 1024u
#define HW 4096u
#define HFRAMES 20000u

typedef struct { Player *target; } Watcher;

int main(void) {
    h_start();
    Player *slab = (Player *)malloc(HP * sizeof(Player));     /* the players' blocks, from one slab */
    Player **players = (Player **)malloc(HP * sizeof(Player *));
    Watcher *ws = (Watcher *)malloc(HW * sizeof(Watcher));
    if (!slab || !players || !ws) abort();
    for (uint32_t i = 0; i < HP; i++) {
        Player *p = &slab[i];
        memset(p, 0, sizeof *p);
        player_init_header(p);
        p->hp = 1000; p->x = (int32_t)i;
        players[i] = p;
    }
    static volatile size_t vlen = HP;
    size_t players_len = vlen; (void)players_len;            /* the list's length, not known when compiling */
    for (uint32_t i = 0; i < HW; i++) {
        Player *t = players[i & (HP - 1)];
        retain(t);
        ws[i].target = t;
    }
    uint64_t acc = 0;
    for (uint32_t f = 0; f < HFRAMES; f++) {
        for (uint32_t i = 0; i < HW; i++) {
            uint32_t j = (3u * i + f) & (HP - 1);
#if VARIANT != V_PLAIN
            if (UNLIKELY(j >= players_len)) e_stop("index out of bounds");   /* bounds check, as today */
#endif
            Player *t = players[j];
            retain(t);                                       /* ws[i].target = shares mut t */
            Player *old = ws[i].target;
            ws[i].target = t;
            release(old);
            acc += (uint64_t)(uint32_t)t->x;
        }
    }
    uint64_t sum = acc;
    for (uint32_t i = 0; i < HW; i++) sum += (uint64_t)(uint32_t)ws[i].target->x;
    h_report(NAME, sum);
    return 0;
}
