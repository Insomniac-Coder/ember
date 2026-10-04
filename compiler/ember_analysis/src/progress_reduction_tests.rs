//! Actual-MIR proof, fallback, and mutation regressions.
use super::*;
use ember_types::TyKind;

fn span() -> Span {
    Span::new(ember_span::FileId(0), 0, 1)
}
fn stmt(l: u32, r: Rvalue) -> Stmt {
    Stmt::new(
        StmtKind::Assign {
            place: Place::local(LocalId(l)),
            rvalue: r,
        },
        span(),
    )
}
fn block(stmts: Vec<Stmt>, terminator: Terminator) -> BasicBlock {
    BasicBlock {
        stmts,
        terminator,
        terminator_span: span(),
    }
}
fn cast(l: u32, to: Ty) -> Rvalue {
    Rvalue::Cast {
        kind: CastKind::Numeric,
        operand: copy(LocalId(l)),
        to,
    }
}
fn binary(op: BinOp, l: u32, r: u32) -> Rvalue {
    Rvalue::BinaryOp {
        op,
        lhs: copy(LocalId(l)),
        rhs: copy(LocalId(r)),
    }
}
fn fixture() -> (TypeTable, CommonTypes, Body) {
    let (mut types, c) = TypeTable::new();
    let reference = types.intern(TyKind::Ref {
        inner: c.usize,
        mutable: true,
    });
    let tys = [
        c.void, c.str_, c.i64, c.usize, c.usize, c.bool_, c.i64, reference, c.char_, c.u32, c.i64,
        c.i64, c.bool_,
    ];
    let locals = tys
        .into_iter()
        .enumerate()
        .map(|(i, ty)| LocalDecl {
            ty,
            kind: if i == 0 {
                LocalKind::Return
            } else if i <= 2 {
                LocalKind::Arg
            } else {
                LocalKind::Temp
            },
            name: None,
            span: span(),
        })
        .collect();
    let blocks = vec![
        block(
            vec![stmt(3, Rvalue::Use(int(0, c.usize)))],
            Terminator::Goto(BasicBlockId(1)),
        ),
        block(
            vec![],
            Terminator::Call {
                func: FuncRef::Builtin {
                    which: Builtin::SpanLen,
                    arg_ty: c.str_,
                },
                args: vec![copy(LocalId(1))],
                dest: Place::local(LocalId(4)),
                next: BasicBlockId(2),
            },
        ),
        block(
            vec![stmt(5, binary(BinOp::Lt, 3, 4))],
            Terminator::SwitchInt {
                discr: copy(LocalId(5)),
                targets: vec![(0, BasicBlockId(5))],
                otherwise: BasicBlockId(3),
            },
        ),
        block(
            vec![
                stmt(6, cast(3, c.i64)),
                stmt(
                    7,
                    Rvalue::Ref {
                        place: Place::local(LocalId(3)),
                        mutable: true,
                    },
                ),
            ],
            Terminator::Call {
                func: FuncRef::Builtin {
                    which: Builtin::StrCharNext,
                    arg_ty: c.str_,
                },
                args: vec![copy(LocalId(1)), copy(LocalId(7))],
                dest: Place::local(LocalId(8)),
                next: BasicBlockId(4),
            },
        ),
        block(
            vec![
                stmt(9, cast(8, c.u32)),
                stmt(10, cast(9, c.i64)),
                stmt(11, binary(BinOp::BitXor, 6, 10)),
                Stmt::new(
                    StmtKind::CheckedBinaryOp {
                        dest: Place::local(LocalId(2)),
                        overflow: Place::local(LocalId(12)),
                        op: BinOp::Add,
                        lhs: copy(LocalId(2)),
                        rhs: copy(LocalId(11)),
                    },
                    span(),
                ),
            ],
            Terminator::Assert {
                cond: copy(LocalId(12)),
                expected: false,
                msg: AssertKind::Overflow(BinOp::Add),
                next: BasicBlockId(1),
                span: span(),
            },
        ),
        block(vec![], Terminator::Return),
    ];
    let b = Body {
        name: "semantic_walk".into(),
        symbol: ember_branding::mangled("semantic_walk"),
        is_unsafe: false,
        abi: None,
        overflow: ember_types::OverflowPolicy::Panic,
        fp: ember_types::FpMode::Strict,
        inline: ember_mir::InlineHint::default(),
        export_thread_policy: ember_mir::ExportThreadPolicy::Any,
        locals,
        blocks,
        arg_count: 2,
        param_modes: vec![ember_mir::ParameterMode::Borrow; 2],
        span: span(),
        borrows: None,
        sources: vec![],
        is_lambda: false,
        emit_if_used: false,
        borrowed_params: vec![LocalId(1), LocalId(2)],
        call_argument_bindings: vec![],
        for_iterators: vec![],
        callable_regions: None,
        closure_environment: None,
        closure_captures_by_move: false,
        class_owner: None,
        class_virtual_slot: None,
        is_abstract: false,
        is_extern_declaration: false,
        ffi_counted: None,
        mut_self: false,
        elided_accesses: vec![],
        hoisted_accesses: vec![],
        uncounted_handles: vec![],
        removed_checks: vec![],
        restrict_views: false,
    };
    (types, c, b)
}
fn extra(b: &mut Body, ty: Ty) -> LocalId {
    let l = LocalId(b.locals.len() as u32);
    b.locals.push(LocalDecl {
        ty,
        kind: LocalKind::Temp,
        name: None,
        span: span(),
    });
    l
}
fn accept(b: Body, t: &TypeTable, c: &CommonTypes) -> (Body, ProgressReductionCertificates) {
    let mut bodies = vec![b];
    let certificates = version_progress_reductions_all(&mut bodies, t, c);
    assert_eq!(certificates.count(), 1);
    assert!(verify_progress_reductions_all(&bodies, t, c, &certificates).is_empty());
    (bodies.remove(0), certificates)
}
fn decline(b: Body, t: &TypeTable, c: &CommonTypes) {
    let before = b.clone();
    let mut bodies = vec![b];
    assert!(version_progress_reductions_all(&mut bodies, t, c).is_empty());
    assert!(same_body(&before, &bodies[0]));
}

#[test]
fn general_term_families_not_names_or_xor_recognition() {
    for family in 0..5 {
        let (t, c, mut b) = fixture();
        b.name = format!("walk_{family}");
        b.symbol = ember_branding::mangled(&format!("walk_{family}"));
        b.blocks[4].stmts[2] = stmt(
            11,
            match family {
                0 => Rvalue::Use(int(7, c.i64)),
                1 => Rvalue::Use(copy(LocalId(10))),
                2 => binary(BinOp::BitAnd, 6, 10),
                3 => binary(BinOp::BitOr, 6, 10),
                _ => binary(BinOp::BitXor, 6, 10),
            },
        );
        let before = b.clone();
        let (after, certs) = accept(b, &t, &c);
        assert!(same_stmt(
            &after.blocks[4].stmts[3],
            &before.blocks[4].stmts[3]
        ));
        assert!(same_term(
            &after.blocks[4].terminator,
            &before.blocks[4].terminator
        ));
        let p = plan(&certs.entries[0].original, &t, &c, 1).unwrap();
        let clone_check = before.blocks.len() + p.region.iter().position(|&x| x == 4).unwrap();
        assert!(matches!(
            after.blocks[clone_check].stmts[3].kind,
            StmtKind::Assign {
                rvalue: Rvalue::BinaryOp { op: BinOp::Add, .. },
                ..
            }
        ));
        assert!(matches!(
            after.blocks[clone_check].terminator,
            Terminator::Goto(_)
        ));
    }
}

#[test]
fn flag_lifetime_markers_preserved_in_both_paths() {
    let (t, c, mut b) = fixture();
    b.blocks[4].terminator = Terminator::Assert {
        cond: copy(LocalId(12)),
        expected: false,
        msg: AssertKind::Overflow(BinOp::Add),
        next: BasicBlockId(6),
        span: span(),
    };
    b.blocks.push(block(
        vec![Stmt::new(StmtKind::StorageDead(LocalId(12)), span())],
        Terminator::Goto(BasicBlockId(1)),
    ));
    let (after, certs) = accept(b, &t, &c);
    let p = plan(&certs.entries[0].original, &t, &c, 1).unwrap();
    let cloned =
        certs.entries[0].original.blocks.len() + p.region.iter().position(|&x| x == 6).unwrap();
    assert!(matches!(
        after.blocks[6].stmts[0].kind,
        StmtKind::StorageDead(LocalId(12))
    ));
    assert!(matches!(
        after.blocks[cloned].stmts[0].kind,
        StmtKind::StorageDead(LocalId(12))
    ));
}

#[test]
fn pure_branch_dag_may_skip_its_single_reduction() {
    let (t, c, mut b) = fixture();
    let branch = extra(&mut b, c.bool_);
    b.blocks[0].stmts.push(stmt(
        branch.0,
        Rvalue::Use(Operand::Const(Const::Bool(true))),
    ));
    if let Terminator::Call { next, .. } = &mut b.blocks[3].terminator {
        *next = BasicBlockId(6);
    }
    b.blocks.push(block(
        vec![],
        Terminator::SwitchInt {
            discr: copy(branch),
            targets: vec![(0, BasicBlockId(7))],
            otherwise: BasicBlockId(4),
        },
    ));
    b.blocks
        .push(block(vec![], Terminator::Goto(BasicBlockId(1))));
    let (_after, _certificates) = accept(b, &t, &c);
}

#[test]
fn physical_block_order_does_not_define_fast_entry() {
    let (t, c, b) = fixture();
    let permutation = [0usize, 4, 2, 1, 3, 5];
    let mut shuffled = b.clone();
    for (old, data) in b.blocks.iter().enumerate() {
        let mut data = data.clone();
        crate::loop_version::retarget(&mut data.terminator, |bb| {
            BasicBlockId(permutation[bb.0 as usize] as u32)
        });
        shuffled.blocks[permutation[old]] = data;
    }
    let (after, certs) = accept(shuffled, &t, &c);
    let p = plan(&certs.entries[0].original, &t, &c, 4).unwrap();
    assert_ne!(p.header, *p.region.first().unwrap());
    assert!(guard_semantics(
        &after,
        &c,
        &p,
        certs.entries[0].original.blocks.len(),
        certs.entries[0].original.locals.len()
    ));
}

#[test]
fn whole_body_initialization_lattice_budget_declines_before_analysis() {
    let (t, c, mut b) = fixture();
    // Individually legal counts, but their checked product exceeds the cap.
    while b.locals.len() < 1025 {
        extra(&mut b, c.i64);
    }
    while b.blocks.len() < 256 {
        b.blocks.push(block(vec![], Terminator::Return));
    }
    assert!(!budget(&b));
    decline(b, &t, &c);
}

#[test]
fn proof_batch_cannot_lose_entries_or_bodies() {
    let (t, c, b) = fixture();
    let (after, mut certs) = accept(b, &t, &c);
    assert!(!verify_progress_reductions_all(&[], &t, &c, &certs).is_empty());
    certs.entries.clear(); // internal inverse; no public filtering API exists
    assert!(!verify_progress_reductions_all(&[after], &t, &c, &certs).is_empty());
}

#[test]
fn nonnegative_mask_bounds_unknown_signed_operand() {
    let (t, c, mut b) = fixture();
    let unknown = extra(&mut b, c.i64);
    b.blocks[0].stmts.push(stmt(
        unknown.0,
        Rvalue::Use(Operand::Const(Const::Int {
            value: u128::MAX,
            ty: c.i64,
        })),
    ));
    b.blocks[4].stmts[2] = stmt(
        11,
        Rvalue::BinaryOp {
            op: BinOp::BitAnd,
            lhs: copy(LocalId(6)),
            rhs: copy(unknown),
        },
    );
    let (_after, _certificates) = accept(b, &t, &c);
}

#[test]
fn reaching_zero_all_entries_and_point_kills() {
    let (t, c, b) = fixture();
    for kind in 0..5 {
        let mut b = b.clone();
        let temp = extra(&mut b, c.usize);
        let later = match kind {
            0 => stmt(3, Rvalue::Use(int(1, c.usize))),
            1 => Stmt::new(StmtKind::StorageDead(LocalId(3)), span()),
            2 => Stmt::new(StmtKind::StorageLive(LocalId(3)), span()),
            3 => stmt(temp.0, Rvalue::Use(Operand::Move(Place::local(LocalId(3))))),
            _ => Stmt::new(
                StmtKind::Drop {
                    place: Place::local(LocalId(3)),
                    flag: None,
                    scope_end: true,
                },
                span(),
            ),
        };
        b.blocks[0].stmts.push(later);
        decline(b, &t, &c);
    }
    let mut two = b;
    let test = extra(&mut two, c.bool_);
    two.blocks[0] = block(
        vec![stmt(test.0, Rvalue::Use(Operand::Const(Const::Bool(true))))],
        Terminator::SwitchInt {
            discr: copy(test),
            targets: vec![(0, BasicBlockId(6))],
            otherwise: BasicBlockId(7),
        },
    );
    for _ in 0..2 {
        two.blocks.push(block(
            vec![stmt(3, Rvalue::Use(int(0, c.usize)))],
            Terminator::Goto(BasicBlockId(1)),
        ));
    }
    let (_after, _certificates) = accept(two.clone(), &t, &c);
    two.blocks[7].stmts[0] = stmt(3, Rvalue::Use(int(1, c.usize)));
    decline(two, &t, &c);
}

#[test]
fn preentry_call_write_after_zero_is_not_a_zero_reaching_definition() {
    let (t, c, mut b) = fixture();
    b.blocks[0].terminator = Terminator::Call {
        func: FuncRef::Direct {
            symbol: ember_branding::mangled("unknown_usize"),
            latebound: false,
        },
        args: vec![],
        dest: Place::local(LocalId(3)),
        next: BasicBlockId(1),
    };
    // All facts say initialized, but the call can return a nonzero/nonboundary
    // cursor. This is logical MIR only; no decoder or forged view is run.
    decline(b, &t, &c);
}

#[test]
fn alias_stale_carry_mixed_sign_unsupported_edges_and_metadata_decline() {
    let (t, c, b) = fixture();
    let mut alias = b.clone();
    let r = extra(&mut alias, b.local(LocalId(7)).ty);
    alias.blocks[0].stmts.push(stmt(
        r.0,
        Rvalue::Ref {
            place: Place::local(LocalId(3)),
            mutable: true,
        },
    ));
    decline(alias, &t, &c);
    let mut stale = b.clone();
    let carried = extra(&mut stale, c.i64);
    stale.blocks[3]
        .stmts
        .insert(0, stmt(carried.0, Rvalue::Use(copy(LocalId(2)))));
    stale.blocks[3]
        .stmts
        .insert(1, stmt(2, Rvalue::Use(int(1, c.i64))));
    if let StmtKind::CheckedBinaryOp { lhs, .. } = &mut stale.blocks[4].stmts[3].kind {
        *lhs = copy(carried);
    }
    decline(stale, &t, &c);
    let mut negative = b.clone();
    negative.blocks[4].stmts[2] = stmt(
        11,
        Rvalue::Use(Operand::Const(Const::Int {
            value: u128::MAX,
            ty: c.i64,
        })),
    );
    decline(negative, &t, &c);
    let mut no_step = b.clone();
    if let Terminator::SwitchInt { otherwise, .. } = &mut no_step.blocks[2].terminator {
        *otherwise = BasicBlockId(4);
    }
    decline(no_step, &t, &c);
    let mut inner_cycle = b.clone();
    if let Terminator::Assert { next, .. } = &mut inner_cycle.blocks[4].terminator {
        *next = BasicBlockId(2);
    }
    decline(inner_cycle, &t, &c);
    let mut hook = b.clone();
    hook.blocks[3].terminator = Terminator::Call {
        func: FuncRef::Direct {
            symbol: ember_branding::mangled("observable_effect"),
            latebound: false,
        },
        args: vec![copy(LocalId(7))],
        dest: Place::local(LocalId(8)),
        next: BasicBlockId(4),
    };
    decline(hook, &t, &c);
    let mut cursor_write = b;
    cursor_write.blocks[1].terminator = Terminator::Call {
        func: FuncRef::Builtin {
            which: Builtin::SpanLen,
            arg_ty: c.str_,
        },
        args: vec![copy(LocalId(1))],
        dest: Place::local(LocalId(3)),
        next: BasicBlockId(2),
    };
    decline(cursor_write, &t, &c);
}

#[test]
fn valid_unrelated_fn5_metadata_declines_whole_body() {
    let (mut t, c, mut b) = fixture();
    let value = extra(&mut b, c.i32);
    let ref_ty = t.intern(TyKind::Ref {
        inner: c.i32,
        mutable: true,
    });
    let source = extra(&mut b, ref_ty);
    let binding = extra(&mut b, ref_ty);
    b.locals[binding.0 as usize].kind = LocalKind::User;
    b.blocks[5] = block(
        vec![
            stmt(value.0, Rvalue::Use(int(0, c.i32))),
            stmt(
                source.0,
                Rvalue::Ref {
                    place: Place::local(value),
                    mutable: true,
                },
            ),
            stmt(binding.0, Rvalue::Use(copy(source))),
        ],
        Terminator::Call {
            func: FuncRef::Direct {
                symbol: ember_branding::mangled("fn5_outside"),
                latebound: false,
            },
            args: vec![copy(binding)],
            dest: Place::local(LocalId(0)),
            next: BasicBlockId(6),
        },
    );
    b.blocks.push(block(vec![], Terminator::Return));
    b.call_argument_bindings
        .push(ember_mir::CallArgumentBinding {
            source,
            binding,
            call_block: BasicBlockId(5),
        });
    assert!(ember_mir::verify::verify(&b).is_empty());
    decline(b, &t, &c);
}

#[test]
fn complete_region_rejects_extra_entry_exit_second_step_and_non_dominating_term() {
    let (t, c, b) = fixture();
    let mut entry = b.clone();
    let choose = extra(&mut entry, c.bool_);
    entry.blocks[0].stmts.push(stmt(
        choose.0,
        Rvalue::Use(Operand::Const(Const::Bool(true))),
    ));
    entry.blocks[0].terminator = Terminator::SwitchInt {
        discr: copy(choose),
        targets: vec![(0, BasicBlockId(4))],
        otherwise: BasicBlockId(1),
    };
    decline(entry, &t, &c);
    let mut exit = b.clone();
    let choose = extra(&mut exit, c.bool_);
    exit.blocks[0].stmts.push(stmt(
        choose.0,
        Rvalue::Use(Operand::Const(Const::Bool(true))),
    ));
    if let Terminator::Assert { next, .. } = &mut exit.blocks[4].terminator {
        *next = BasicBlockId(6);
    }
    exit.blocks.push(block(
        vec![],
        Terminator::SwitchInt {
            discr: copy(choose),
            targets: vec![(0, BasicBlockId(5))],
            otherwise: BasicBlockId(1),
        },
    ));
    decline(exit, &t, &c);
    let mut duplicate = b.clone();
    if let Terminator::Call { next, .. } = &mut duplicate.blocks[3].terminator {
        *next = BasicBlockId(6);
    }
    duplicate.blocks.push(block(
        vec![],
        Terminator::Call {
            func: FuncRef::Builtin {
                which: Builtin::StrCharNext,
                arg_ty: c.str_,
            },
            args: vec![copy(LocalId(1)), copy(LocalId(7))],
            dest: Place::local(LocalId(8)),
            next: BasicBlockId(4),
        },
    ));
    decline(duplicate, &t, &c);
    let mut branch = b;
    let choose = extra(&mut branch, c.bool_);
    branch.blocks[0].stmts.extend([
        stmt(choose.0, Rvalue::Use(Operand::Const(Const::Bool(true)))),
        stmt(11, Rvalue::Use(int(0, c.i64))),
    ]);
    branch.blocks[4].stmts.remove(2);
    if let Terminator::Call { next, .. } = &mut branch.blocks[3].terminator {
        *next = BasicBlockId(6);
    }
    branch.blocks.push(block(
        vec![],
        Terminator::SwitchInt {
            discr: copy(choose),
            targets: vec![(0, BasicBlockId(4))],
            otherwise: BasicBlockId(7),
        },
    ));
    branch.blocks.push(block(
        vec![stmt(11, Rvalue::Use(int(1, c.i64)))],
        Terminator::Goto(BasicBlockId(4)),
    ));
    decline(branch, &t, &c);
}

#[test]
fn uninitialized_accumulator_and_term_read_decline() {
    let (t, c, mut b) = fixture();
    b.locals[2].kind = LocalKind::User;
    b.arg_count = 1;
    b.param_modes.truncate(1);
    b.borrowed_params.truncate(1);
    decline(b, &t, &c);
    let (t, c, mut b) = fixture();
    b.blocks[4]
        .stmts
        .insert(2, Stmt::new(StmtKind::StorageDead(LocalId(10)), span()));
    decline(b, &t, &c);
}

#[test]
fn snapshot_verifier_rejects_guard_edge_clone_span_and_both_paths_mutated() {
    let (t, c, b) = fixture();
    let (after, certs) = accept(b, &t, &c);
    let p = plan(&certs.entries[0].original, &t, &c, 1).unwrap();
    let first = certs.entries[0].original.blocks.len() + p.region.len();
    let check_clone =
        certs.entries[0].original.blocks.len() + p.region.iter().position(|&x| x == 4).unwrap();
    for corruption in 0..8 {
        let mut bad = after.clone();
        match corruption {
            0 => {
                if let StmtKind::Assign {
                    rvalue: Rvalue::BinaryOp { op, .. },
                    ..
                } = &mut bad.blocks[first + 4].stmts[1].kind
                {
                    *op = BinOp::Add;
                }
            }
            1 => {
                if let Terminator::SwitchInt { targets, .. } = &mut bad.blocks[first + 2].terminator
                {
                    targets[0].1 = BasicBlockId(check_clone as u32);
                }
            }
            2 => bad.blocks[check_clone].stmts[3].span = Span::new(ember_span::FileId(0), 2, 3),
            3 => {
                for i in [4, check_clone] {
                    if let StmtKind::CheckedBinaryOp { rhs, .. } = &mut bad.blocks[i].stmts[3].kind
                    {
                        *rhs = int(0, c.i64);
                    } else if let StmtKind::Assign {
                        rvalue: Rvalue::BinaryOp { rhs, .. },
                        ..
                    } = &mut bad.blocks[i].stmts[3].kind
                    {
                        *rhs = int(0, c.i64);
                    }
                }
            }
            4 => bad.blocks[0].stmts[0] = stmt(3, Rvalue::Use(int(1, c.usize))),
            5 => bad.blocks[0].terminator = Terminator::Goto(BasicBlockId(1)),
            6 => {
                let r = extra(&mut bad, after.local(LocalId(7)).ty);
                bad.blocks[0].stmts.push(stmt(
                    r.0,
                    Rvalue::Ref {
                        place: Place::local(LocalId(3)),
                        mutable: true,
                    },
                ));
            }
            _ => bad.removed_checks[0].span = Span::new(ember_span::FileId(0), 3, 4),
        }
        assert!(
            !verify_progress_reductions_all(&[bad], &t, &c, &certs).is_empty(),
            "corruption {corruption}"
        );
    }
}

#[test]
fn independent_guard_checker_rejects_bad_construction_without_replaying_apply() {
    let (t, c, b) = fixture();
    let (after, certs) = accept(b, &t, &c);
    let before = &certs.entries[0].original;
    let p = plan(before, &t, &c, 1).unwrap();
    let first = before.blocks.len() + p.region.len();
    assert!(guard_semantics(
        &after,
        &c,
        &p,
        before.blocks.len(),
        before.locals.len()
    ));
    for defect in 0..5 {
        // Simulate a bad constructor output. The assertion exercises the
        // independent semantic checker directly, without expected=apply().
        let mut bad = after.clone();
        match defect {
            0 => {
                if let StmtKind::Assign {
                    rvalue: Rvalue::BinaryOp { op, .. },
                    ..
                } = &mut bad.blocks[first + 4].stmts[3].kind
                {
                    *op = BinOp::Lt;
                }
            }
            1 => {
                if let StmtKind::Assign {
                    rvalue: Rvalue::BinaryOp { lhs, rhs, .. },
                    ..
                } = &mut bad.blocks[first + 4].stmts[1].kind
                {
                    std::mem::swap(lhs, rhs);
                }
            }
            2 => {
                let remaining = match &bad.blocks[first + 2].stmts[1].kind {
                    StmtKind::Assign { place, .. } => place.local,
                    _ => unreachable!(),
                };
                if let StmtKind::Assign {
                    rvalue: Rvalue::BinaryOp { rhs, .. },
                    ..
                } = &mut bad.blocks[first + 4].stmts[2].kind
                {
                    *rhs = copy(remaining);
                }
            }
            3 => {
                if let Terminator::SwitchInt { otherwise, .. } =
                    &mut bad.blocks[first + 1].terminator
                {
                    *otherwise = BasicBlockId(first as u32);
                }
            }
            _ => {
                if let StmtKind::Assign {
                    rvalue: Rvalue::BinaryOp { rhs, .. },
                    ..
                } = &mut bad.blocks[first + 4].stmts[2].kind
                {
                    *rhs = int(0, c.u64);
                }
            }
        }
        assert!(
            !guard_semantics(&bad, &c, &p, before.blocks.len(), before.locals.len()),
            "constructor defect {defect}"
        );
    }
}

// This interpreter exercises ONLY the newly introduced pure guards. It never
// creates a buffer/view pointer or simulates successful UTF8 decoder calls.
// u64 operations model emitted unsigned arithmetic; i128 is an independent
// product+sum oracle for bounded fixed cases, not the quotient implementation.
fn guard_decision(b: &Body, p: &Plan, length: u64, cursor: u64, acc: i64) -> (bool, bool) {
    let first = b.blocks.len() - 5;
    let fast = first - p.region.len() + p.region.iter().position(|&x| x == p.header).unwrap();
    let mut values = BTreeMap::<LocalId, u64>::from([(p.cursor, cursor), (p.acc, acc as u64)]);
    let mut read_acc = false;
    fn operand(
        o: &Operand,
        v: &BTreeMap<LocalId, u64>,
        view: LocalId,
        length: u64,
        acc: LocalId,
        read: &mut bool,
    ) -> u64 {
        match o {
            Operand::Const(Const::Int { value, .. }) => *value as u64,
            Operand::Const(Const::Bool(x)) => u64::from(*x),
            Operand::Copy(place)
                if place.local == view && place.projection == vec![Projection::Field(1)] =>
            {
                length
            }
            Operand::Copy(place) if place.projection.is_empty() => {
                if place.local == acc {
                    *read = true;
                }
                v[&place.local]
            }
            _ => panic!("unsupported guard operand"),
        }
    }
    let mut bb = first;
    for _ in 0..5 {
        for s in &b.blocks[bb].stmts {
            let StmtKind::Assign { place, rvalue } = &s.kind else {
                panic!("non-pure guard")
            };
            let mut get = |o| operand(o, &values, p.view, length, p.acc, &mut read_acc);
            let value = match rvalue {
                Rvalue::Use(o) | Rvalue::Cast { operand: o, .. } => get(o),
                Rvalue::BinaryOp { op, lhs, rhs } => {
                    let a = get(lhs);
                    let r = get(rhs);
                    match op {
                        BinOp::Add => a.wrapping_add(r),
                        BinOp::Sub => a.wrapping_sub(r),
                        BinOp::Div => a / r,
                        BinOp::BitAnd => a & r,
                        BinOp::BitXor => a ^ r,
                        BinOp::Le => u64::from(a <= r),
                        BinOp::Lt => u64::from(a < r),
                        BinOp::Ne => u64::from(a != r),
                        _ => panic!("unsupported guard op"),
                    }
                }
                _ => panic!("unsupported guard rvalue"),
            };
            values.insert(place.local, value);
        }
        let Terminator::SwitchInt {
            discr,
            targets,
            otherwise,
        } = &b.blocks[bb].terminator
        else {
            panic!("guard branch")
        };
        let value = operand(discr, &values, p.view, length, p.acc, &mut read_acc);
        let to = if value == 0 {
            targets[0].1.0 as usize
        } else {
            otherwise.0 as usize
        };
        if to == p.header {
            return (false, read_acc);
        }
        if to == fast {
            return (true, read_acc);
        }
        bb = to;
    }
    panic!("guard exceeded exact recipe extent")
}

#[test]
fn guard_fixed_extremes_matches_wider_product_sum_and_no_run_skips_acc_read() {
    let (t, c, mut original) = fixture();
    original.blocks[4].stmts[2] = stmt(11, Rvalue::Use(int(7, c.i64)));
    let (after, certs) = accept(original, &t, &c);
    let p = plan(&certs.entries[0].original, &t, &c, 1).unwrap();
    let rows = [
        (i64::MIN, 1u64),
        (i64::MIN, u64::MAX),
        (i64::MAX, 1),
        (i64::MAX, 0),
        (-1, 1),
        (-1, LIMIT / 7),
        (0, LIMIT / 7),
        (0, LIMIT / 7 + 1),
        (i64::MAX - 7, 1),
        (i64::MAX - 7, 2),
        (42, u32::MAX as u64),
    ];
    for (a, length) in rows {
        let (fast, read) = guard_decision(&after, &p, length, 0, a);
        let expected = length != 0
            && length <= LIMIT
            && (a as i128) + (length as i128) * 7 <= i64::MAX as i128;
        assert_eq!(fast, expected, "A={a} L={length}");
        if length == 0 || length > LIMIT {
            assert!(!read);
        }
    }
    assert_eq!(guard_decision(&after, &p, 4, 5, 0), (false, false));
    assert_eq!(guard_decision(&after, &p, 4, 4, i64::MAX), (false, false));
    // Native32-sized L and a separately u64-sized proof bound: no narrowing.
    let (t, c, mut original) = fixture();
    original.blocks[4].stmts[2] = stmt(11, Rvalue::Use(int(LIMIT, c.i64)));
    let (after, certs) = accept(original, &t, &c);
    let p = plan(&certs.entries[0].original, &t, &c, 1).unwrap();
    assert_eq!(guard_decision(&after, &p, 3, 0, i64::MIN).0, false); // R*U exceeds u64; quotient stays exact.
    assert_eq!(guard_decision(&after, &p, 2, 0, i64::MIN).0, true);
    let (t, c, mut original) = fixture();
    original.blocks[4].stmts[2] = stmt(11, Rvalue::Use(int(0, c.i64)));
    let (after, certs) = accept(original, &t, &c);
    let p = plan(&certs.entries[0].original, &t, &c, 1).unwrap();
    assert_eq!(guard_decision(&after, &p, 1, 0, i64::MAX), (true, false)); // no divide or A read when U=0.
}

#[test]
fn symbolic_xor_upper_uses_old_statement_point_offset() {
    let (t, c, b) = fixture();
    let (after, certs) = accept(b, &t, &c);
    let p = plan(&certs.entries[0].original, &t, &c, 1).unwrap();
    for length in [1u64, 4, 127, 65536, u32::MAX as u64, LIMIT] {
        let upper = ((length - 1) as u128 + 0x10ffff).min(LIMIT as u128);
        for a in [i64::MIN, -1, 0, i64::MAX] {
            let expected = (a as i128) + (length as i128) * (upper as i128) <= LIMIT as i128;
            assert_eq!(guard_decision(&after, &p, length, 0, a).0, expected);
        }
    }
}

#[test]
fn old_offset_zero_and_post_step_offset_one_have_different_credit() {
    let (t, c, mut original) = fixture();
    original.blocks[4].stmts[2] = stmt(11, Rvalue::Use(copy(LocalId(6))));
    let (old, certs) = accept(original.clone(), &t, &c);
    let p = plan(&certs.entries[0].original, &t, &c, 1).unwrap();
    assert_eq!(p.upper, Upper::BeforeStep);
    assert_eq!(guard_decision(&old, &p, 1, 0, i64::MAX), (true, false));
    let post = extra(&mut original, c.i64);
    original.blocks[4]
        .stmts
        .insert(2, stmt(post.0, cast(3, c.i64)));
    original.blocks[4].stmts[3] = stmt(11, Rvalue::Use(copy(post)));
    let (after, certs) = accept(original, &t, &c);
    let p = plan(&certs.entries[0].original, &t, &c, 1).unwrap();
    assert_eq!(p.upper, Upper::Length);
    assert_eq!(guard_decision(&after, &p, 1, 0, i64::MAX), (false, true));
}
