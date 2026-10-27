/* salvage_w2v_dispatch.c (M2, workload "w2v"): version the walk on the function value actually
   stored. The compiler knows every function the program may store in `on_visit` (R12, whole program)
   and each one's summary (R5): `count_visit` changes no node, `prune` may end nodes. At the start of
   each walk the code word is compared with the functions that change no node; when it is one of them
   the walk keeps views and pays nothing per step (model_w2v_views' walk), otherwise it runs the
   rule's walk (model_w2v_move's: header test and count move per step).

   Safety: the views walk runs only while the stored function is one whose summary changes no node, so
   no call in it can end a node and the views stay valid ([WK-18] last sentence, the case the spec
   already allows for a program that never stores `prune`). The test holds for the whole walk because
   no function the program may store writes `on_visit` (its summary says so); a program where one did
   would test once per step instead, still nothing on the nodes. Cost: one load and one compare per walk
   (1,000,000 per pass here), and a second copy of the walk's code. */
#define FORM_DISPATCH
#include "model_w2v.c"
