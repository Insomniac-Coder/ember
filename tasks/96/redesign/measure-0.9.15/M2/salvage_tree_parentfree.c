/* salvage_tree_parentfree.c (M2, workload "tree"): a parent link proved by [WK-17] condition 1 holds
   no weak count. Everything else is model_tree.c as written: the index links stay counted, every open
   of an index link still tests the header, and the parent read stays a plain load.

   Safety argument (the report, section "tree", gives it in full): a weak count exists only to keep its
   target's block readable for a later header test. [WK-17] condition 1 proves that while a node lives
   its parent owns it, so every read of the link finds a live target and is compiled with no header
   test; no test ever reads the link's target after that target ends. The link is never handed over or
   copied (a copy is `weakref n.parent`, a new link that takes its own count). During a drop chain no
   code runs that reads the link (condition 1 needs a class with no `drop` of its own). So a counted
   parent link and an uncounted one give the same reads, and the count only moves numbers no read
   uses. This is run 3's `set by` link (uncounted, `RUN3/combined-v4.md` D7) under R11's own proof. */
#define PARENT_UNCOUNTED
#include "model_tree.c"
