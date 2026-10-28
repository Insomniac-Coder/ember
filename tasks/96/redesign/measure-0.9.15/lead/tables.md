| cb_event program | runs | median ms | peak KB | x plain (same compiler) | spread | ok |
|---|---|---|---|---|---|---|
| plain_cb_event | 15 | 162.3 | 3452 | 1.0 | 21.7% | yes |
| plain_cb_event_msvc | 15 | 161.7 | 3724 | 1.0 | 19.6% | yes |
| model_cb_event | 15 | 254.0 | 3456 | 1.565 | 2.8% | yes |
| model_cb_event_msvc | 15 | 189.9 | 3736 | 1.174 | 2.1% | yes |
| model_cb_event_nothread | 15 | 197.2 | 3456 | 1.215 | 2.8% | yes |
| model_cb_event_nothread_msvc | 15 | 167.2 | 3736 | 1.034 | 3.4% | yes |
| salvage_cb_event_fold | 15 | 199.2 | 3456 | 1.227 | 1.9% | yes |
| salvage_cb_event_fold_msvc | 15 | 200.3 | 3732 | 1.239 | 2.7% | yes |
| salvage_cb_event_teb | 15 | 160.5 | 3452 | 0.989 | 2.6% | yes |
| salvage_cb_event_teb_msvc | 15 | 178.9 | 3736 | 1.106 | 3.2% | yes |
| rust_cb_event | 15 | 159.4 | 3784 | 0.982 | 17.5% | yes |

| cb_once program | runs | median ms | peak KB | x plain (same compiler) | spread | ok |
|---|---|---|---|---|---|---|
| plain_cb_once | 15 | 149.5 | 3488 | 1.0 | 4.8% | yes |
| plain_cb_once_msvc | 15 | 143.0 | 3760 | 1.0 | 4.7% | yes |
| model_cb_once | 15 | 235.5 | 3508 | 1.576 | 8.0% | yes |
| model_cb_once_msvc | 15 | 176.3 | 3796 | 1.233 | 6.9% | yes |
| model_cb_once_nothread | 15 | 220.2 | 3516 | 1.473 | 36.8% | yes |
| model_cb_once_nothread_msvc | 15 | 174.2 | 3796 | 1.218 | 6.6% | yes |
| salvage_cb_once_genmark | 15 | 235.1 | 3516 | 1.573 | 16.0% | yes |
| salvage_cb_once_genmark_msvc | 15 | 176.9 | 3796 | 1.237 | 19.3% | yes |
| salvage_cb_once_nomark | 15 | 226.7 | 3512 | 1.517 | 14.4% | yes |
| salvage_cb_once_nomark_msvc | 15 | 169.1 | 3796 | 1.183 | 4.7% | yes |
| rust_cb_once | 15 | 1143.2 | 3836 | 7.648 | 2.8% | yes |

| cb_parse program | runs | median ms | peak KB | x plain (same compiler) | spread | ok |
|---|---|---|---|---|---|---|
| plain_cb_parse | 5 | 83.0 | 15012 | 1.0 | 3.3% | yes |
| plain_cb_parse_msvc | 5 | 101.1 | 15256 | 1.0 | 2.5% | yes |
| model_cb_parse | 5 | 89.8 | 14976 | 1.082 | 1.3% | yes |
| model_cb_parse_msvc | 5 | 102.9 | 15256 | 1.018 | 1.4% | yes |
| model_cb_parse_nothread | 5 | 88.6 | 14984 | 1.067 | 2.4% | yes |
| model_cb_parse_nothread_msvc | 5 | 105.8 | 15292 | 1.047 | 2.7% | yes |
| salvage_cb_parse_fold | 5 | 89.4 | 14996 | 1.077 | 1.1% | yes |
| salvage_cb_parse_fold_msvc | 5 | 105.2 | 15264 | 1.04 | 1.1% | yes |
| salvage_cb_parse_teb | 5 | 88.9 | 14972 | 1.072 | 2.1% | yes |
| salvage_cb_parse_teb_msvc | 5 | 110.6 | 15288 | 1.094 | 2.5% | yes |
| rust_cb_parse | 5 | 84.5 | 15316 | 1.019 | 2.9% | yes |

| churn program | runs | median ms | peak KB | x plain (same compiler) | spread | ok |
|---|---|---|---|---|---|---|
| plain_churn_malloc | 5 | 1155.4 | 17896 | 1.0 | 1.2% | yes |
| plain_churn_malloc_msvc | 5 | 1202.1 | 18172 | 1.0 | 2.1% | yes |
| plain_churn_pool | 5 | 289.6 | 13744 | 0.251 | 3.5% | yes |
| plain_churn_pool_msvc | 5 | 355.8 | 14024 | 0.296 | 0.9% | yes |
| model_churn | 5 | 384.2 | 15796 | 0.333 | 2.2% | yes |
| model_churn_msvc | 5 | 396.2 | 16072 | 0.33 | 5.8% | yes |
| salvage_churn_chunkhdr | 5 | 295.7 | 13740 | 0.256 | 3.3% | yes |
| salvage_churn_chunkhdr_msvc | 5 | 355.4 | 14024 | 0.296 | 3.1% | yes |
| rust_churn box | 5 | 1196.9 | 18284 | 1.036 | 1.2% | yes |
| rust_churn pbox | 5 | 335.8 | 17564 | 0.291 | 2.4% | yes |
| rust_churn slab | 5 | 376.1 | 22256 | 0.325 | 2.0% | yes |

| hotshare program | runs | median ms | peak KB | x plain (same compiler) | spread | ok |
|---|---|---|---|---|---|---|
| plain_hotshare | 5 | 18.2 | 3532 | 1.0 | 1.7% | yes |
| plain_hotshare_msvc | 5 | 29.9 | 3812 | 1.0 | 3.2% | yes |
| model_hotshare | 5 | 60.9 | 3540 | 3.351 | 4.8% | yes |
| model_hotshare_msvc | 5 | 61.3 | 3820 | 2.049 | 6.4% | yes |
| diag_hotshare_nocompare | 5 | 61.3 | 3536 | 3.369 | 5.3% | yes |
| diag_hotshare_nocompare_msvc | 5 | 60.7 | 3816 | 2.028 | 1.7% | yes |
| model_hotshare_ts | 5 | 655.7 | 3536 | 36.061 | 0.3% | yes |
| model_hotshare_ts_msvc | 5 | 659.6 | 3820 | 22.031 | 0.7% | yes |
| diag_hotshare_ts_nocompare | 5 | 658.6 | 3536 | 36.225 | 0.9% | yes |
| diag_hotshare_ts_nocompare_msvc | 5 | 659.0 | 3816 | 22.011 | 0.7% | yes |
| salvage_hotshare_fold | 5 | 60.9 | 3540 | 3.352 | 6.2% | yes |
| salvage_hotshare_fold_msvc | 5 | 61.3 | 3820 | 2.049 | 2.0% | yes |
| salvage_hotshare_wide | 5 | 61.1 | 3540 | 3.36 | 7.6% | yes |
| salvage_hotshare_wide_msvc | 5 | 61.2 | 3816 | 2.044 | 1.2% | yes |
| salvage_hotshare_ts_wide | 5 | 660.9 | 3540 | 36.351 | 1.1% | yes |
| salvage_hotshare_ts_wide_msvc | 5 | 658.5 | 3820 | 21.995 | 1.7% | yes |
| rust_hotshare_rc | 5 | 126.6 | 3904 | 6.965 | 2.2% | yes |
| rust_hotshare_arc | 5 | 703.8 | 3908 | 38.71 | 0.2% | yes |

| lists program | runs | median ms | peak KB | x plain (same compiler) | spread | ok |
|---|---|---|---|---|---|---|
| plain_lists_malloc | 5 | 750.3 | 21324 | 1.0 | 2.1% | yes |
| plain_lists_malloc_msvc | 5 | 754.1 | 21612 | 1.0 | 0.5% | yes |
| plain_lists_arena | 5 | 216.8 | 17052 | 0.289 | 1.5% | yes |
| plain_lists_arena_msvc | 5 | 241.3 | 17396 | 0.32 | 1.5% | yes |
| model_lists | 5 | 278.2 | 19140 | 0.371 | 3.6% | yes |
| model_lists_msvc | 5 | 272.9 | 19432 | 0.362 | 3.5% | yes |
| model_lists_inl | 5 | 247.9 | 19128 | 0.33 | 1.3% | yes |
| model_lists_inl_msvc | 5 | 250.7 | 19444 | 0.332 | 0.9% | yes |
| salvage_lists_known | 5 | 240.1 | 17112 | 0.32 | 1.5% | yes |
| salvage_lists_known_msvc | 5 | 244.0 | 17376 | 0.324 | 1.1% | yes |
| salvage_lists_chunkhdr | 5 | 243.1 | 17100 | 0.324 | 1.1% | yes |
| salvage_lists_chunkhdr_msvc | 5 | 254.0 | 17392 | 0.337 | 2.7% | yes |
| rust_lists vec | 5 | 780.1 | 21704 | 1.04 | 1.9% | yes |
| rust_lists avec | 5 | 232.5 | 19512 | 0.31 | 0.9% | yes |

| respan program | runs | median ms | peak KB | x plain (same compiler) | spread | ok |
|---|---|---|---|---|---|---|
| plain_respan | 5 | 96.0 | 3476 | 1.0 | 2.4% | yes |
| plain_respan_msvc | 5 | 214.2 | 3752 | 1.0 | 3.2% | yes |
| model_respan | 5 | 96.7 | 3480 | 1.008 | 1.5% | yes |
| model_respan_msvc | 5 | 254.3 | 3756 | 1.188 | 1.9% | yes |
| model_respan_words | 5 | 96.7 | 3476 | 1.007 | 5.7% | yes |
| model_respan_words_msvc | 5 | 213.1 | 3756 | 0.995 | 1.8% | yes |
| salvage_respan_flag | 5 | 103.6 | 3472 | 1.079 | 1.2% | yes |
| salvage_respan_flag_msvc | 5 | 224.5 | 3756 | 1.048 | 1.1% | yes |
| rust_respan repoint | 5 | 153.2 | 3852 | 1.597 | 2.2% | yes |
| rust_respan plain | 5 | 130.5 | 3852 | 1.36 | 3.2% | yes |
| plain_respan_rtlen | 5 | 96.1 | 3480 | 1.0 | 2.2% | yes |
| plain_respan_rtlen_msvc | 5 | 249.5 | 3756 | 1.0 | 2.5% | yes |
| model_respan_rtlen | 5 | 96.6 | 3480 | 1.005 | 1.6% | yes |
| model_respan_rtlen_msvc | 5 | 253.4 | 3756 | 1.016 | 1.5% | yes |
| model_respan_words_rtlen | 5 | 98.1 | 3480 | 1.021 | 1.8% | yes |
| model_respan_words_rtlen_msvc | 5 | 244.6 | 3756 | 0.98 | 2.3% | yes |
| salvage_respan_flag_rtlen | 5 | 105.6 | 3476 | 1.099 | 1.6% | yes |
| salvage_respan_flag_rtlen_msvc | 5 | 258.9 | 3756 | 1.038 | 1.4% | yes |

| retarget program | runs | median ms | peak KB | x plain (same compiler) | spread | ok |
|---|---|---|---|---|---|---|
| plain_retarget | 15 | 218.5 | 53280 | 1.0 | 10.7% | yes |
| plain_retarget_msvc | 15 | 214.0 | 53572 | 1.0 | 13.8% | yes |
| model_retarget | 15 | 253.7 | 55332 | 1.161 | 7.5% | yes |
| model_retarget_msvc | 15 | 263.3 | 55624 | 1.231 | 8.6% | yes |
| diag_retarget_nocompare | 15 | 250.6 | 55332 | 1.147 | 11.9% | yes |
| diag_retarget_nocompare_msvc | 15 | 260.1 | 55624 | 1.215 | 14.6% | yes |
| model_retarget_ts | 15 | 260.4 | 55336 | 1.192 | 18.3% | yes |
| model_retarget_ts_msvc | 15 | 266.8 | 55624 | 1.247 | 27.0% | yes |
| diag_retarget_ts_nocompare | 15 | 263.2 | 55336 | 1.205 | 6.5% | yes |
| diag_retarget_ts_nocompare_msvc | 15 | 266.8 | 55624 | 1.247 | 37.7% | yes |
| salvage_retarget_fold | 15 | 250.6 | 55336 | 1.147 | 20.9% | yes |
| salvage_retarget_fold_msvc | 15 | 266.3 | 55624 | 1.245 | 27.0% | yes |
| salvage_retarget_wide | 15 | 251.5 | 55336 | 1.151 | 6.8% | yes |
| salvage_retarget_wide_msvc | 15 | 263.5 | 55624 | 1.232 | 25.7% | yes |
| salvage_retarget_ts_wide | 15 | 263.4 | 55336 | 1.206 | 12.5% | yes |
| salvage_retarget_ts_wide_msvc | 15 | 268.2 | 55624 | 1.253 | 13.0% | yes |
| rust_retarget_rc | 15 | 336.5 | 61928 | 1.54 | 10.1% | yes |
| rust_retarget_arc | 15 | 347.0 | 61932 | 1.588 | 15.9% | yes |

| samevar_one program | runs | median ms | peak KB | x plain (same compiler) | spread | ok |
|---|---|---|---|---|---|---|
| plain_samevar one | 5 | 325.8 | 3496 | 1.0 | 2.2% | yes |
| plain_samevar_msvc one | 5 | 328.7 | 3780 | 1.0 | 1.0% | yes |
| model_samevar one | 5 | 89.6 | 3496 | 0.275 | 4.1% | yes |
| model_samevar_msvc one | 5 | 99.3 | 3784 | 0.302 | 1.5% | yes |
| salvage_samevar_generic one | 5 | 325.7 | 3496 | 1.0 | 1.7% | yes |
| salvage_samevar_generic_msvc one | 5 | 330.9 | 3784 | 1.007 | 2.2% | yes |
| rust_samevar one | 5 | 88.8 | 3864 | 0.273 | 0.9% | yes |

| samevar_two program | runs | median ms | peak KB | x plain (same compiler) | spread | ok |
|---|---|---|---|---|---|---|
| plain_samevar two | 15 | 100.1 | 3492 | 1.0 | 4.0% | yes |
| plain_samevar_msvc two | 15 | 129.0 | 3784 | 1.0 | 1.8% | yes |
| plain_samevar_noalias two | 15 | 90.9 | 3492 | 0.908 | 11.7% | yes |
| plain_samevar_noalias_msvc two | 15 | 110.3 | 3784 | 0.855 | 5.3% | yes |
| model_samevar two | 15 | 109.9 | 3496 | 1.098 | 3.4% | yes |
| model_samevar_msvc two | 15 | 119.8 | 3784 | 0.929 | 2.3% | yes |
| rust_samevar two | 15 | 109.9 | 3864 | 1.098 | 4.0% | yes |

| selfremove program | runs | median ms | peak KB | x plain (same compiler) | spread | ok |
|---|---|---|---|---|---|---|
| plain_selfremove | 5 | 186.7 | 4180 | 1.0 | 1.6% | yes |
| plain_selfremove_msvc | 5 | 214.2 | 4456 | 1.0 | 2.9% | yes |
| model_selfremove | 5 | 209.0 | 4264 | 1.12 | 1.8% | yes |
| model_selfremove_msvc | 5 | 284.9 | 4528 | 1.33 | 1.0% | yes |
| salvage_selfremove_hoist | 5 | 207.8 | 4264 | 1.113 | 5.2% | yes |
| salvage_selfremove_hoist_msvc | 5 | 280.3 | 4524 | 1.309 | 1.3% | yes |
| rust_selfremove | 5 | 274.1 | 4524 | 1.468 | 2.0% | yes |

| this_mix program | runs | median ms | peak KB | x plain (same compiler) | spread | ok |
|---|---|---|---|---|---|---|
| plain_this_mix | 5 | 76.5 | 4484 | 1.0 | 3.3% | yes |
| plain_this_mix_msvc | 5 | 65.1 | 4768 | 1.0 | 1.7% | yes |
| plain_this_mix_noalias | 5 | 60.9 | 4488 | 0.796 | 1.8% | yes |
| plain_this_mix_noalias_msvc | 5 | 60.1 | 4768 | 0.924 | 1.7% | yes |
| model_this_mix | 5 | 77.4 | 4484 | 1.012 | 3.5% | yes |
| model_this_mix_msvc | 5 | 65.5 | 4764 | 1.007 | 5.6% | yes |
| salvage_this_mix_typeproof | 5 | 61.9 | 4488 | 0.81 | 3.3% | yes |
| salvage_this_mix_typeproof_msvc | 5 | 59.9 | 4768 | 0.92 | 2.2% | yes |
| salvage_this_mix_hoist | 5 | 153.8 | 4488 | 2.011 | 2.3% | yes |
| salvage_this_mix_hoist_msvc | 5 | 154.9 | 4768 | 2.381 | 0.2% | yes |
| rust_this_mix | 5 | 61.0 | 4824 | 0.798 | 2.3% | yes |

| this_tally program | runs | median ms | peak KB | x plain (same compiler) | spread | ok |
|---|---|---|---|---|---|---|
| plain_this_tally | 5 | 289.5 | 3968 | 1.0 | 2.1% | yes |
| plain_this_tally_msvc | 5 | 288.0 | 4248 | 1.0 | 0.6% | yes |
| plain_this_tally_noalias | 5 | 51.9 | 3964 | 0.179 | 3.5% | yes |
| plain_this_tally_noalias_msvc | 5 | 81.9 | 4252 | 0.284 | 2.9% | yes |
| model_this_tally | 5 | 91.2 | 3968 | 0.315 | 2.6% | yes |
| model_this_tally_msvc | 5 | 114.4 | 4256 | 0.397 | 1.6% | yes |
| salvage_this_tally_typeproof | 5 | 96.7 | 3968 | 0.334 | 1.2% | yes |
| salvage_this_tally_typeproof_msvc | 5 | 114.6 | 4252 | 0.398 | 3.8% | yes |
| salvage_this_tally_hoist | 5 | 81.2 | 3964 | 0.28 | 4.5% | yes |
| salvage_this_tally_hoist_msvc | 5 | 119.1 | 4252 | 0.414 | 1.6% | yes |
| rust_this_tally | 5 | 85.5 | 4304 | 0.295 | 3.8% | yes |

| this_tick program | runs | median ms | peak KB | x plain (same compiler) | spread | ok |
|---|---|---|---|---|---|---|
| plain_this_tick | 5 | 271.4 | 4220 | 1.0 | 1.4% | yes |
| plain_this_tick_msvc | 5 | 253.2 | 4508 | 1.0 | 1.8% | yes |
| plain_this_tick_noalias | 5 | 271.6 | 4220 | 1.001 | 0.6% | yes |
| plain_this_tick_noalias_msvc | 5 | 255.1 | 4508 | 1.008 | 2.0% | yes |
| model_this_tick | 5 | 267.3 | 4348 | 0.985 | 1.6% | yes |
| model_this_tick_msvc | 5 | 164.1 | 4632 | 0.648 | 2.1% | yes |
| salvage_this_tick_typeproof | 5 | 266.0 | 4348 | 0.98 | 2.2% | yes |
| salvage_this_tick_typeproof_msvc | 5 | 166.2 | 4640 | 0.657 | 2.2% | yes |
| salvage_this_tick_hoist | 5 | 107.6 | 4344 | 0.396 | 9.9% | yes |
| salvage_this_tick_hoist_msvc | 5 | 157.7 | 4640 | 0.623 | 2.6% | yes |
| rust_this_tick | 5 | 254.3 | 4732 | 0.937 | 2.9% | yes |

| tree program | runs | median ms | peak KB | x plain (same compiler) | spread | ok |
|---|---|---|---|---|---|---|
| plain_tree | 15 | 445.7 | 56748 | 1.0 | 4.9% | yes |
| plain_tree_msvc | 15 | 440.0 | 57052 | 1.0 | 23.4% | yes |
| plain_same_tree | 15 | 1015.9 | 87160 | 2.279 | 6.9% | yes |
| plain_same_tree_msvc | 15 | 1021.2 | 87340 | 2.321 | 9.2% | yes |
| model_tree | 15 | 1151.0 | 87056 | 2.582 | 9.3% | yes |
| model_tree_msvc | 15 | 1135.3 | 87336 | 2.58 | 11.5% | yes |
| salvage_tree_parentfree | 15 | 1067.7 | 87160 | 2.395 | 7.2% | yes |
| salvage_tree_parentfree_msvc | 15 | 1096.9 | 87340 | 2.493 | 10.7% | yes |
| rust_tree_rc | 15 | 1659.8 | 118456 | 3.724 | 5.9% | yes |
| rust_tree_arena | 15 | 311.5 | 53564 | 0.699 | 10.1% | yes |

| trees program | runs | median ms | peak KB | x plain (same compiler) | spread | ok |
|---|---|---|---|---|---|---|
| plain_trees_malloc | 15 | 939.6 | 36836 | 1.0 | 10.4% | yes |
| plain_trees_malloc_msvc | 15 | 938.0 | 37112 | 1.0 | 11.8% | yes |
| plain_trees_arena | 15 | 191.0 | 28212 | 0.203 | 10.2% | yes |
| plain_trees_arena_msvc | 15 | 178.3 | 28488 | 0.19 | 6.3% | yes |
| model_trees | 15 | 394.0 | 44732 | 0.419 | 21.8% | yes |
| model_trees_msvc | 15 | 418.0 | 45008 | 0.446 | 21.2% | yes |
| model_trees_inl | 15 | 352.8 | 44736 | 0.375 | 21.9% | yes |
| model_trees_inl_msvc | 15 | 258.7 | 45008 | 0.276 | 16.4% | yes |
| model_trees_inl_nowalk | 15 | 240.6 | 44732 | 0.256 | 12.7% | yes |
| model_trees_inl_nowalk_msvc | 15 | 260.3 | 45008 | 0.278 | 16.9% | yes |
| salvage_trees_nofree | 15 | 191.0 | 28212 | 0.203 | 18.1% | yes |
| salvage_trees_nofree_msvc | 15 | 179.5 | 28488 | 0.191 | 15.9% | yes |
| salvage_trees_chunkhdr | 15 | 253.9 | 28140 | 0.27 | 18.3% | yes |
| salvage_trees_chunkhdr_msvc | 15 | 187.3 | 28420 | 0.2 | 15.2% | yes |
| rust_trees box | 15 | 991.6 | 37248 | 1.055 | 10.1% | yes |
| rust_trees abox | 15 | 375.7 | 45108 | 0.4 | 25.7% | yes |
| rust_trees bref | 15 | 209.3 | 28584 | 0.223 | 15.3% | yes |
| rust_trees index | 15 | 185.2 | 37672 | 0.197 | 12.8% | yes |

| w2v program | runs | median ms | peak KB | x plain (same compiler) | spread | ok |
|---|---|---|---|---|---|---|
| plain_w2v | 15 | 959.2 | 54932 | 1.0 | 24.7% | yes |
| plain_w2v_msvc | 15 | 984.2 | 55220 | 1.0 | 22.7% | yes |
| plain_w2v_novisit | 15 | 479.2 | 54932 | 0.5 | 22.3% | yes |
| plain_w2v_novisit_msvc | 15 | 455.5 | 55216 | 0.463 | 12.7% | yes |
| plain_same_w2v | 15 | 1356.6 | 84348 | 1.414 | 28.4% | yes |
| plain_same_w2v_msvc | 15 | 1334.8 | 84636 | 1.356 | 9.7% | yes |
| model_w2v_move | 15 | 1414.1 | 84332 | 1.474 | 18.5% | yes |
| model_w2v_move_msvc | 15 | 1439.9 | 84640 | 1.463 | 22.5% | yes |
| model_w2v_move_drop | 15 | 1418.3 | 84356 | 1.479 | 12.2% | yes |
| model_w2v_move_drop_msvc | 15 | 1409.3 | 84640 | 1.432 | 14.0% | yes |
| model_w2v_region | 15 | 1180.9 | 84372 | 1.231 | 15.2% | yes |
| model_w2v_region_msvc | 15 | 1277.7 | 84636 | 1.298 | 12.8% | yes |
| model_w2v_views | 15 | 1365.6 | 84344 | 1.424 | 22.3% | yes |
| model_w2v_views_msvc | 15 | 1357.3 | 84632 | 1.379 | 13.6% | yes |
| model_w2v_views_novisit | 15 | 656.1 | 84352 | 0.684 | 10.0% | yes |
| model_w2v_views_novisit_msvc | 15 | 642.8 | 84640 | 0.653 | 11.0% | yes |
| salvage_w2v_freeonly | 15 | 1279.4 | 84360 | 1.334 | 12.9% | yes |
| salvage_w2v_freeonly_msvc | 15 | 1288.4 | 84632 | 1.309 | 10.4% | yes |
| salvage_w2v_dispatch | 15 | 1360.2 | 84352 | 1.418 | 25.2% | yes |
| salvage_w2v_dispatch_msvc | 15 | 1372.2 | 84636 | 1.394 | 10.3% | yes |
| salvage_w2v_hoist | 15 | 1175.4 | 84376 | 1.225 | 15.1% | yes |
| salvage_w2v_hoist_msvc | 15 | 1092.8 | 84636 | 1.11 | 19.3% | yes |
| rust_w2v | 15 | 1019.5 | 52324 | 1.063 | 12.5% | yes |
| rust_w2v_rc | 15 | 2654.4 | 116260 | 2.767 | 9.7% | yes |

| weakview program | runs | median ms | peak KB | x plain (same compiler) | spread | ok |
|---|---|---|---|---|---|---|
| plain_weakview | 15 | 53.4 | 12032 | 1.0 | 12.3% | yes |
| plain_weakview_msvc | 15 | 54.3 | 12312 | 1.0 | 10.5% | yes |
| model_weakview | 15 | 544.6 | 12308 | 10.193 | 2.4% | yes |
| model_weakview_msvc | 15 | 546.1 | 12592 | 10.062 | 4.7% | yes |
| model_weakview_onethread | 15 | 53.2 | 12312 | 0.996 | 10.8% | yes |
| model_weakview_onethread_msvc | 15 | 56.5 | 12588 | 1.04 | 6.1% | yes |
| salvage_weakview_plaindata | 15 | 61.1 | 12312 | 1.144 | 2.5% | yes |
| salvage_weakview_plaindata_msvc | 15 | 64.6 | 12588 | 1.191 | 6.0% | yes |
| rust_weakview_arc | 15 | 701.0 | 12920 | 13.119 | 1.8% | yes |
| rust_weakview_rc | 15 | 64.3 | 12928 | 1.204 | 10.2% | yes |

