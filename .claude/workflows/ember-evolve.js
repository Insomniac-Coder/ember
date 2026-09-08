export const meta = {
  name: 'ember-evolve',
  description: 'Analyse the Ember language spec across 7 lenses, adversarially challenge every proposal, and synthesise the next RFC change-set',
  phases: [
    { title: 'Analyse', detail: 'seven independent lenses over the spec, the ADRs, the errata and the compiler' },
    { title: 'Challenge', detail: 'two adversaries: already-rejected/duplicate, and soundness/implementability' },
    { title: 'Synthesise', detail: 'one editor turns the survivors into a numbered RFC change-set with exact spec deltas' },
  ],
}

// Re-runnable across spec revisions. Override any of these via the Workflow tool's `args`:
//   { root, from, to, spec, out, notes }
// e.g. Workflow({name: 'ember-evolve', args: {from: '0.4', to: '0.5'}})
const CFG = (typeof args === 'object' && args) || {}
const ROOT = CFG.root || '/home/user/ember'
const FROM = CFG.from || '0.3'
const TO = CFG.to || '0.4'
const SPEC = CFG.spec || ROOT + '/docs/spec-source/ember-' + FROM + '.md'
const OUT = CFG.out || ROOT + '/docs/evolution/RFC-v' + TO + '.md'

// Version-specific ground truth. Replace via args.notes on each new revision — a stale
// note here is worse than none, because seven agents will treat it as fact.
const NOTES = CFG.notes || [
  '  - v0.3 was authored from v0.2 and DOES NOT carry four rulings the owner already made in docs/spec-errata.md.',
  '    v0.3 has therefore silently reverted them. This is known; the reconciliation lens owns it. Do not spend',
  '    your budget re-discovering it unless you are the reconciliation lens.',
].join('\n')

const PREAMBLE = [
  'You are helping evolve EMBER, a systems programming language, from specification v' + FROM + ' toward a v' + TO + ' that is the best possible expression of its owner\'s vision.',
  '',
  'THE VISION, in the owner\'s own words:',
  '  "blazing fast speed of C, readability and syntax similar to Python, and the memory safety of Rust,',
  '   while being LESS ANNOYING than Rust" ... "I want to create a version which will be loved by people."',
  'Reuse of, and interoperability with, existing C and C++ code is an explicit first-class goal.',
  '',
  'READ THESE (absolute paths):',
  '  ' + SPEC + '   <- THE v' + FROM + ' SPECIFICATION. This is the primary input.',
  '  ' + ROOT + '/docs/spec/                       <- the spec split by part, as the compiler was actually built against.',
  '  ' + ROOT + '/docs/DECISIONS.md                <- 9 owner-confirmed ADRs. These are SETTLED unless you have a decisive new argument.',
  '  ' + ROOT + '/docs/spec-errata.md              <- 7 errata; 4 already ruled on by the owner.',
  '  ' + ROOT + '/docs/HANDOFF.md                  <- state of the implementation, invariants, traps already paid for.',
  '  ' + ROOT + '/compiler/  ' + ROOT + '/runtime/  ' + ROOT + '/tests/  ' + ROOT + '/examples/',
  '',
  'GROUND TRUTH ABOUT THE PROJECT STATE:',
  '  - Phase 0 of 9 is complete: a working end-to-end compiler (Rust) emitting C11, 146 tests passing.',
  '    Lexer, full v1 grammar parser, AST, HIR, typeck, MIR with a verifier, C backend, driver, C11 runtime.',
  NOTES,
  '  - The reference workload is RageV, a Windows C++ game engine (Vulkan 1.3 + OpenGL 4.5, sparse-set ECS,',
  '    render graph, C# scripting via a function-pointer table). You MUST NOT read, build or modify any RageV code.',
  '',
  'THE BAR FOR A FINDING. This document has been reviewed many times and is unusually careful about its own',
  'trade-offs. Shallow observations are worse than useless here because they cost the owner attention.',
  'Before you propose ANYTHING:',
  '  1. Read Part 0\'s rejected-alternatives table (17 rows). If your proposal is one of those rows, it is',
  '     already rejected WITH A STATED REASON. You may only re-propose it if you directly defeat that reason.',
  '  2. Read Part XXII.2 (non-goals) and the "Considered and rejected for 0.3" paragraph in the Part 0 change log.',
  '  3. Read docs/DECISIONS.md. The 9 ADRs are owner-confirmed. Contradicting one requires an explicit,',
  '     decisive argument, and you must say which ADR you are reopening.',
  'A finding that ignores steps 1-3 will be killed in the Challenge phase and will have wasted everyone\'s time.',
  '',
  'WHAT A GOOD FINDING LOOKS LIKE:',
  '  - It names the exact rule id(s) or section it concerns ([BRW-5], Part IX.8, XVI.7, ...).',
  '  - Its evidence is a CONCRETE Ember or C++ code sample showing the problem, not an abstract worry.',
  '  - Its proposal is specific enough that a compiler engineer could implement it without asking a question.',
  '  - It states what the change COSTS: implementation work, what it breaks, what it complicates.',
  '  - It is honest about being a judgement call when it is one.',
  '',
  'Return AT MOST 10 findings, ranked with the most valuable first. Ten excellent findings beat thirty adequate',
  'ones. If your lens only yields four things worth the owner\'s time, return four. Do NOT pad.',
  'Do NOT modify any file in the repository; you are read-only.',
].join('\n')

const LENSES = [
  {
    key: 'RECON',
    label: 'reconciliation',
    title: 'Specification / implementation reconciliation',
    brief: [
      'You own the drift between the three sources of truth: the v' + FROM + ' document, the older split spec in docs/spec/,',
      'and the compiler that actually exists.',
      '',
      'Do this concretely:',
      '  1. Diff v' + FROM + ' against docs/spec/ part by part. Catalogue EVERY substantive difference, not just the ones',
      '     the 0.3 change log advertises. The change log claims 10 changes; verify that claim and find what it missed.',
      '  2. For each of the 7 entries in docs/spec-errata.md, determine whether v' + FROM + ' carries the ruling or reverts it.',
      '     Four were decided by the owner (ERR-001, ERR-005, ERR-006, ERR-007). Reverting a decided ruling is a BLOCKER.',
      '  3. For each of the 9 ADRs, verify v' + FROM + ' is consistent with it.',
      '  4. Read the compiler (lexer, parser, diag codes, typeck) and find places where the shipped behaviour and',
      '     v' + FROM + ' disagree. docs/HANDOFF.md lists load-bearing invariants; check v' + FROM + ' does not violate them.',
      '  5. Check internal consistency of v' + FROM + ' itself: rules referenced but never defined, error codes used but not',
      '     in the registry, examples that contradict the rules they illustrate, sections the change log promises',
      '     (X.1.1, [EFF-9], [EFF-11], [EFF-12], [DIA-11], [CLI-3], diagnostic shapes B11 and S1) that may be',
      '     missing, truncated, or inconsistent with each other.',
      '',
      'Your output is the highest-priority part of this whole exercise: it is the patch list that makes v' + TO + '',
      'and the compiler agree again. Be exhaustive within your 10 slots; group mechanical items into one finding',
      'each where they share a cause, and list every instance inside that finding\'s spec_delta.',
    ].join('\n'),
  },
  {
    key: 'ERGO',
    label: 'ergonomics',
    title: 'Ergonomics: the "less annoying than Rust" claim',
    brief: [
      'This is the central claim of the language. Test it adversarially rather than accepting it.',
      '',
      'Take real code shapes that make Rust programmers suffer and write them in Ember using ONLY what Part VII,',
      'VIII, IX and XIII actually specify. Then judge honestly whether Ember is genuinely better or has merely',
      'moved the pain:',
      '  - a tree or graph with parent pointers; an observer list; a callback registry',
      '  - a struct holding a borrow of something it does not own; two views into one buffer',
      '  - a linked structure being mutated while iterated',
      '  - builder chains; partial moves out of a struct; self-referential state machines',
      '  - error handling across layers: [ERR-2] "?" with From conversion, and what happens when E types multiply',
      '  - closures that capture, escape, and outlive their frame ([CLO-*], Part VI.5)',
      '  - generic code with [IFC-*] bounds and where clauses, and what the diagnostics say when a bound is unmet',
      '',
      'Pay specific attention to:',
      '  - [LT-1] region elision rule 3 (intersection) and @borrows: is it really more permissive than Rust in',
      '    practice, or does it just fail later and less legibly?',
      '  - [LT-6]: no named lifetimes in v1. Find the concrete programs this makes UNWRITABLE, not merely awkward.',
      '    This is the biggest ergonomic bet in the language; the owner deserves to know exactly what it costs.',
      '  - Cell/RefCell (IX.7) as the value-world escape hatch: is the ladder complete, or are there rungs missing?',
      '  - Whether the "class" escape hatch quietly becomes the path of least resistance, so that everyone writes',
      '    classes, and the value world - where the performance lives - goes unused. That would be a design failure',
      '    of the vision even though every individual rule is sound.',
      '  - Diagnostics as ergonomics: [PHIL-8] says rejection is correct only when a safe expression of the same',
      '    intent exists AND the diagnostic names it. Audit Part XIX.6 diagnostic shapes against that promise.',
    ].join('\n'),
  },
  {
    key: 'PYTH',
    label: 'readability',
    title: 'Python readability and syntax coherence',
    brief: [
      'The promise is "readability and syntax similar to Python". Audit Part II, Part III, Part VI and Appendix A',
      'against it, from the point of view of a competent Python or C# programmer meeting Ember for the first time.',
      '',
      'Specifically:',
      '  - ADR-002 names a real footgun and lets it stand: with block scoping plus [GRM-4] ("x = expr declares',
      '    when x is not in scope"), a MISSPELLED name inside a nested block silently declares a fresh local and',
      '    the outer variable is never written. In a language whose premise is that the compiler catches such',
      '    things, this is Python\'s oldest bug preserved on purpose. The ADR says "a lint could recover most of',
      '    it" but no lint is specified. Work out the best available answer and specify it properly.',
      '  - ADR-005 chose [] for generics over <>. Trace the consequences the ADR admits (an IndexOrInstantiate node',
      '    unresolved until name resolution, no pre-resolution folding, worse messages for indexing errors) and',
      '    check the grammar and Part XVIII actually handle every ambiguity. Look for cases the ADR did not foresee.',
      '  - Indentation plus the [LEX-*] rules: line continuation, multi-line calls, nested closures, match arms,',
      '    and decorators/attributes. Write awkward-but-realistic code and see whether it stays readable.',
      '  - Where does Ember LOOK like Python but BEHAVE differently? Every such place is a trap for the exact',
      '    audience the syntax was chosen to attract. Enumerate them and say what mitigates each.',
      '  - Consistency of the surface: are there two ways to say one thing, or one keyword doing two jobs?',
      '    Check "mut" (parameter mode vs receiver vs local), "ref", "owned", "let", "pub(read)", "with", "defer".',
      '  - Appendix A is what most people will read first. Judge it as marketing AND as a reference.',
    ].join('\n'),
  },
  {
    key: 'FFI',
    label: 'interop',
    title: 'C/C++ interoperability and code reuse',
    brief: [
      'The owner explicitly asked for improvements here. Part XVI is the biggest part of the spec (19KB); Part XXI',
      'is the integration plan; [BLD-FFI-*] covers build integration. Judge the whole story as an engineer who has',
      'a large existing C++ codebase and wants to adopt Ember INCREMENTALLY without a rewrite.',
      '',
      'Cover at minimum:',
      '  - C import ([FFI-6..16]): headers, macros (function-like and object-like), bitfields, unions, anonymous',
      '    structs, va_args, inline functions, static inline, flexible array members, packed structs, enums whose',
      '    underlying type is implementation-defined, and const-correctness inference.',
      '  - C++ import (XVI.7, [FFI-17..20]): the thunk approach. Templates and template instantiation, STL types',
      '    crossing the boundary (std::string, std::vector, std::unique_ptr, std::shared_ptr, std::function),',
      '    overload sets, default arguments, namespaces, references vs pointers, RAII objects whose destructor must',
      '    run on the C++ side, virtual dispatch, multiple inheritance, ABI stability across compilers, and',
      '    "no C++ subclassing from Ember" (Part XXII.2) - is that non-goal survivable for a real engine?',
      '  - Ownership across the boundary: [FFI-1] contracts, overlays (XVI.4), Retained[T], ForeignBox[T],',
      '    Callback[F], [FFI-22] foreign threads, and what happens to Ember\'s RC and exclusivity invariants when',
      '    a foreign thread or a foreign owner is in the picture.',
      '  - Layout verification [FFI-5], the .embind cache (XVI.5), and [BLD-FFI-1]\'s compile_commands.json path.',
      '  - Exporting Ember to C (XVI.10) and embedding ember_rt: is the embedding story good enough that an engine',
      '    can host Ember the way it hosts C#?',
      '',
      'Also ask the question the spec does not: what is the ADOPTION RAMP? A team with 500k lines of C++ needs a',
      'first day that works. What is missing from Part XVI to make day one succeed - a binding generator UX, a',
      'diagnostic when a header will not import, a way to vendor an overlay, a story for header-only libraries?',
      'Findings that make real C/C++ reuse dramatically easier are the highest value you can produce.',
    ].join('\n'),
  },
  {
    key: 'SOUND',
    label: 'soundness',
    title: 'Soundness of the safety model',
    brief: [
      'Ember claims "no undefined behaviour outside unsafe". Attack that claim. Your job is to find the holes,',
      'not to admire the design.',
      '',
      'Look hardest at the seams, which is where safety models actually break:',
      '  - The 0.3 enforcement ladder [PHIL-8]/[PHIL-9] and the I.4 table: is every row actually true? Find a',
      '    guarantee the table claims is "always statically proven" that in fact is not.',
      '  - Class exclusivity [EXC-1..3] interacting with: re-entrancy, drop running arbitrary code, virtual',
      '    dispatch, closures capturing handles, and handles loaded from memory.',
      '  - RefCell [CELL-5..10] + Ref/RefMut as view types + with-blocks + panics + drop order. And [CELL-9]:',
      '    the shipping profile compiles the checks out and a violation becomes UB. Does that contradict [PRF-1]',
      '    ("profiles must not change semantics") - which [DSJ-8] invokes to forbid exactly this pattern for',
      '    assume_disjoint? Check whether 0.3 introduced an internal inconsistency between [CELL-9] and [DSJ-8].',
      '  - assert_disjoint / assume_disjoint (IX.8): [DSJ-2] says the proof is carried by the value. Try to defeat',
      '    it - aliasing the returned views, reborrowing, storing them in a struct, passing them to a function that',
      '    reassigns, overlapping via SoA columns or arena views, zero-length spans, spans of ZSTs.',
      '  - Arenas [ARN-*] + [LT-4] + drop: can an arena be reset or dropped while a view into it is live?',
      '  - Generational handles [HND-1]/[GPU-1]: generation wraparound at 12 bits, and the shipping profile.',
      '  - unsafe (IX.4, [UNS-*]): is the obligation list complete enough that an "unsafe" author knows the rules?',
      '  - FFI as the biggest hole: [FFI-2] says unknown contracts require unsafe. Is the boundary between',
      '    "contract supplied via overlay, therefore safe" and reality sound? An overlay is a programmer assertion.',
      '  - Send/Sync + [THR-*] + class handles + [FFI-22] trampolines on foreign threads.',
      '',
      'For each hole: give the concrete program that exhibits it, and propose the minimal rule change that closes it.',
    ].join('\n'),
  },
  {
    key: 'PERF',
    label: 'performance',
    title: 'Performance credibility: the "speed of C" claim',
    brief: [
      'Judge whether Ember as specified can actually reach C performance, and where the specification is writing',
      'cheques the implementation cannot cash. ADR-006 fixed a C11 backend for v1 with LLVM deferred to v2.',
      '',
      'Examine:',
      '  - The C backend (Part XVIII.6) as a performance vehicle: what does emitting C11 cost versus emitting LLVM',
      '    IR? Aliasing information (restrict), inlining across translation units, whether the host C compiler can',
      '    see enough to vectorise, debug info, and the MSVC constraints named in XVIII.6.',
      '  - Bounds checks, overflow checks ([TYP-8] per profile), exclusivity checks ([EXC-1], ~2ns per access pair),',
      '    and RefCell borrow state. Add up the cost on a realistic hot loop and compare against C honestly.',
      '  - RC traffic: [RC-2]/[RC-3] elision and [OPT-1] stack promotion. How much retain/release survives in',
      '    idiomatic object-world code, and what does that do to a 60fps frame budget?',
      '  - [SIMD-*], SoA (XII.1), @parallel (XI.4), and whether the specified aliasing facts are strong enough for',
      '    the backend to actually vectorise. This is where assert_disjoint ([DSJ-3]) is supposed to pay off.',
      '  - ADR-001: unsuffixed float literals default to f32. Right for graphics; find where it silently costs',
      '    precision in code ported from C or Python, and whether the spec does enough to warn.',
      '  - @noalloc / @nosync / @noblock ([EFF-5..8]) as the mechanism that makes performance CHECKABLE rather',
      '    than hoped-for. Is the effect system complete enough? What about @nopanic, deferred to v2 - is deferring',
      '    it right, given that panics are what stops a hot loop being provably branch-free?',
      '  - The performance suite (XX.4) and XXI.6\'s pass criteria: are the numbers there actually measurable, and',
      '    is "within 1.05x of C++" a claim the design can support?',
      '',
      'Be quantitative wherever you can. Name the mechanism that must exist for the claim to hold.',
    ].join('\n'),
  },
  {
    key: 'LOVE',
    label: 'adoption',
    title: 'Adoption, tooling and the things that make a language loved',
    brief: [
      'The owner said: "I want to create a version which will be loved by people." Languages are not loved for',
      'their type systems. They are loved for the first hour, the error messages, the tooling, and the feeling',
      'that the language is on your side. Audit Ember for that, and be candid.',
      '',
      'Cover:',
      '  - THE LANGUAGE SERVER. Search the whole spec: there appears to be NO LSP, no editor story, no incremental',
      '    or resilient parsing requirement, no "compile a broken file and still answer questions" contract.',
      '    In 2026 a language without a good LSP on day one is not adopted, however good it is. Work out what the',
      '    spec must say - which IRs must be queryable, what the compiler architecture (Part XVIII) must guarantee',
      '    about incrementality and error recovery - so that a good LSP is POSSIBLE rather than a later rewrite.',
      '    Check whether Part XVIII\'s pipeline is even shaped for it.',
      '  - Error messages: Part XIX.6 specifies diagnostic shapes. Rust\'s reputation rests on these. Judge whether',
      '    the specified shapes reach that bar, and where they fall short. [DIA-11] is a good instinct - extend it.',
      '  - The first hour: install, hello world, a real program, an editor that works, a REPL or playground.',
      '    ember.toml, the CLI (XIX.1), and whether "ember new" / "ember add" / a registry exist at all.',
      '  - Package management and versioning: XIX.2\'s manifest, and what is missing for a real ecosystem.',
      '  - Documentation: XIX.9, docs/book/ deferred to v1.1. Is that too late?',
      '  - Debugging: can you use a normal debugger on Ember code compiled through C? [CG-C-1] #line directives',
      '    exist - is that enough for breakpoints, variable inspection, and stack traces people can read?',
      '  - ADR-009: the name. "Ember" collides with Ember.js, a large, long-established JavaScript framework, and',
      '    contends for the npm name and for search results generally. The owner has flagged this and not decided.',
      '    Give a real recommendation with reasoning - including what it costs to change later versus now.',
      '  - Community and governance: RFC process, versioning and stability policy, what "1.0" promises.',
      '',
      'Rank by what most changes whether a stranger falls in love with this language in their first week.',
    ].join('\n'),
  },
]

const FINDINGS_SCHEMA = {
  type: 'object',
  properties: {
    lens: { type: 'string', description: 'the lens key you were assigned' },
    findings: {
      type: 'array',
      description: 'at most 10, most valuable first',
      items: {
        type: 'object',
        properties: {
          id: { type: 'string', description: 'LENSKEY-n, e.g. ERGO-1' },
          title: { type: 'string', description: 'one line, specific' },
          severity: { type: 'string', enum: ['blocker', 'major', 'minor', 'polish'] },
          spec_location: { type: 'string', description: 'exact Part/section and rule ids affected' },
          problem: { type: 'string', description: 'what is wrong or missing, and why it matters to the vision' },
          evidence: { type: 'string', description: 'concrete code sample, quote, or compiler-source reference proving it' },
          proposal: { type: 'string', description: 'the specific change, implementable without further questions' },
          spec_delta: { type: 'string', description: 'proposed normative text: new/replacement rules with ids, written in the spec voice' },
          cost: { type: 'string', description: 'implementation cost, what it breaks, what it complicates' },
          prior_art_check: { type: 'string', description: 'which Part 0 row, XXII.2 non-goal, ADR or errata entry is adjacent, and why this proposal is not merely re-proposing something already rejected' },
        },
        required: ['id', 'title', 'severity', 'spec_location', 'problem', 'evidence', 'proposal', 'spec_delta', 'cost', 'prior_art_check'],
      },
    },
    lens_verdict: { type: 'string', description: 'your honest overall judgement on this dimension: does v' + FROM + ' deliver the vision here?' },
  },
  required: ['lens', 'findings', 'lens_verdict'],
}

const VERDICT_SCHEMA = {
  type: 'object',
  properties: {
    verdicts: {
      type: 'array',
      items: {
        type: 'object',
        properties: {
          id: { type: 'string' },
          verdict: { type: 'string', enum: ['accept', 'accept-with-changes', 'reject', 'duplicate'] },
          reason: { type: 'string', description: 'be specific and cite the spec where relevant' },
          revision: { type: 'string', description: 'if accept-with-changes, exactly what must change' },
          duplicate_of: { type: 'string', description: 'if duplicate, the id it duplicates' },
        },
        required: ['id', 'verdict', 'reason'],
      },
    },
    cross_cutting: { type: 'string', description: 'conflicts or dependencies BETWEEN proposals that no single finding could see' },
    missed: { type: 'string', description: 'anything important the seven lenses collectively failed to raise' },
  },
  required: ['verdicts', 'cross_cutting', 'missed'],
}

// ---------------------------------------------------------------- Phase 1

phase('Analyse')
log('Seven lenses over Ember v' + FROM + ', the ADRs, the errata and the compiler as it stands.')

const reports = (await parallel(LENSES.map((l) => () =>
  agent(
    PREAMBLE +
    '\n\n================ YOUR LENS ================\n' +
    'Lens key: ' + l.key + '\n' +
    'Lens: ' + l.title + '\n\n' +
    l.brief +
    '\n\n================ OUTPUT ================\n' +
    'Set "lens" to "' + l.key + '". Number your finding ids ' + l.key + '-1, ' + l.key + '-2, ...\n' +
    'Fill every field. An empty or hand-waving prior_art_check is grounds for rejection in the next phase.\n' +
    'Write spec_delta in the specification\'s own normative voice (MUST/SHOULD/MAY, bracketed rule ids), so it\n' +
    'can be pasted into the document with minimal editing.',
    { label: 'analyse:' + l.label, phase: 'Analyse', schema: FINDINGS_SCHEMA }
  )
))).filter(Boolean)

const findings = []
for (const r of reports) {
  const key = (r.lens || '').trim() || 'UNK'
  ;(r.findings || []).forEach((f, i) => {
    const id = f.id && f.id.indexOf(key) === 0 ? f.id : key + '-' + (i + 1)
    findings.push(Object.assign({}, f, { id: id, lens: key }))
  })
}

log('Analyse complete: ' + reports.length + '/' + LENSES.length + ' lenses returned, ' + findings.length + ' findings total.')
if (reports.length < LENSES.length) {
  log('WARNING: ' + (LENSES.length - reports.length) + ' lens(es) failed and produced nothing. Coverage is incomplete.')
}

const lensVerdicts = reports.map((r) => '### Lens ' + r.lens + '\n' + (r.lens_verdict || '')).join('\n\n')
const findingsJson = JSON.stringify(findings, null, 2)

// ---------------------------------------------------------------- Phase 2
// Barrier is justified: both challengers need the COMPLETE finding set to detect
// duplicates across lenses and conflicts between proposals.

phase('Challenge')
log('Two adversaries over all ' + findings.length + ' findings: prior-art/duplication, and soundness/implementability.')

const CHALLENGE_BASE = PREAMBLE +
  '\n\n================ YOUR ROLE ================\n' +
  'You are a CHALLENGER. Seven analysis agents have each proposed changes to Ember v' + FROM + '. Most proposals to a\n' +
  'well-reviewed specification are wrong, redundant, or already considered. Your job is to kill the ones that\n' +
  'do not deserve the owner\'s attention, and to sharpen the ones that do.\n\n' +
  'You must return a verdict for EVERY id in the list. Do not skip any. Do not invent ids.\n' +
  'Be decisive: "accept" means you would defend this to the owner personally.\n\n' +
  'HERE ARE THE LENS-LEVEL VERDICTS:\n' + lensVerdicts +
  '\n\nHERE ARE ALL ' + findings.length + ' FINDINGS AS JSON:\n' + findingsJson + '\n'

const challengers = [
  {
    key: 'prior-art',
    label: 'challenge:prior-art',
    brief: [
      'YOUR SPECIFIC ANGLE: prior art, duplication, and respect for decisions already made.',
      '',
      'Reject a finding when:',
      '  - It re-proposes something in Part 0\'s 17-row rejected-alternatives table without defeating the stated',
      '    reason for the rejection. Quote the row.',
      '  - It contradicts one of the 9 owner-confirmed ADRs without an argument decisive enough to justify',
      '    reopening it. Name the ADR. (Reopening is ALLOWED - but it must be worth the owner\'s time, and the',
      '    finding must say plainly that it is a reopening.)',
      '  - It violates a Part XXII.2 non-goal, or the "Considered and rejected for 0.3" paragraph, without',
      '    addressing why that non-goal should move.',
      '  - The spec already handles it and the agent simply missed the text. Cite the rule that handles it.',
      '  - It is a duplicate of another finding. Mark it "duplicate" and set duplicate_of to the id you consider',
      '    the better-argued of the pair. Findings from different lenses that converge on one underlying issue are',
      '    the most common duplication; catch those.',
      '  - It is generic language-design commentary that could have been written without reading THIS spec.',
      '',
      'Use "accept-with-changes" when the finding identifies something real but frames it wrongly, overreaches,',
      'or must be narrowed to survive contact with a decision already made. State the exact revision.',
      '',
      'Also: judge whether the SET of findings respects the owner\'s vision. A proposal that makes Ember safer by',
      'making it more annoying than Rust has failed the brief, however technically sound it is. Say so.',
    ].join('\n'),
  },
  {
    key: 'soundness',
    label: 'challenge:soundness',
    brief: [
      'YOUR SPECIFIC ANGLE: does the proposal actually WORK, and can it be BUILT?',
      '',
      'For each finding, attack the proposal itself rather than the problem it identifies:',
      '  - Does the proposed rule interact badly with another rule elsewhere in the spec? Name the rule. This is',
      '    the most common failure mode: a local fix that breaks a distant guarantee.',
      '  - Does the spec_delta actually say what the proposal means? Is it precise enough to implement without',
      '    reinterpretation, per the document\'s own standard ("written to be executed against")?',
      '  - Is it implementable in the ACTUAL architecture: a Rust compiler emitting C11 (ADR-006), no LLVM until',
      '    v2, MSVC as a first-class target, the pass pipeline in Part XVIII, and the 9-phase plan in Part XX with',
      '    only Phase 0 complete? A proposal that presumes LLVM, or presumes work from a later phase, must say so.',
      '  - Does it break the invariants in docs/HANDOFF.md, or invalidate work already shipped in compiler/?',
      '  - Does it conflict with ANOTHER finding in this set? Two accepted proposals that cannot both be true is',
      '    a failure you are the only one positioned to catch. Record every such pair in cross_cutting.',
      '  - Is the stated cost honest? Agents systematically underestimate. Correct it in your reason.',
      '  - Does it preserve the ability to reach the vision - C speed, Python readability, Rust safety, less',
      '    annoyance - or does it trade one pillar away to strengthen another without saying so?',
      '',
      'Reject soundly-motivated proposals that do not work. A real problem with a broken fix is still a reject;',
      'say in your reason that the PROBLEM is real, so the editor can carry it forward as an open question.',
    ].join('\n'),
  },
]

const challenges = (await parallel(challengers.map((c) => () =>
  agent(CHALLENGE_BASE + '\n' + c.brief, { label: c.label, phase: 'Challenge', schema: VERDICT_SCHEMA })
))).filter(Boolean)

const byId = {}
for (const f of findings) byId[f.id] = { finding: f, verdicts: [] }
for (const ch of challenges) {
  for (const v of (ch.verdicts || [])) {
    if (byId[v.id]) byId[v.id].verdicts.push(v)
  }
}

const survivors = []
const killed = []
for (const id of Object.keys(byId)) {
  const rec = byId[id]
  const vs = rec.verdicts
  const rejected = vs.filter((v) => v.verdict === 'reject' || v.verdict === 'duplicate')
  if (rejected.length > 0) killed.push({ finding: rec.finding, verdicts: vs })
  else survivors.push({ finding: rec.finding, verdicts: vs })
}

log('Challenge complete: ' + survivors.length + ' survived, ' + killed.length + ' rejected or merged.')

// ---------------------------------------------------------------- Phase 3

phase('Synthesise')

const SYNTH_SCHEMA = {
  type: 'object',
  properties: {
    file_written: { type: 'string', description: 'absolute path of the RFC document you wrote' },
    rfc_count: { type: 'number' },
    headline: { type: 'string', description: 'the 3-6 changes that matter most, one line each' },
    v_next_scope: { type: 'string', description: 'your recommended scope for the next version versus what should wait' },
    open_questions: { type: 'string', description: 'what still needs an owner decision, in the style of Part XXII.1' },
    honest_assessment: { type: 'string', description: 'candid answer: how close is v' + FROM + ' to the owner\'s vision, and what is the single biggest gap?' },
  },
  required: ['file_written', 'rfc_count', 'headline', 'v_next_scope', 'open_questions', 'honest_assessment'],
}

const synth = await agent(
  PREAMBLE +
  '\n\n================ YOUR ROLE ================\n' +
  'You are the EDITOR. Seven lenses proposed changes to Ember v' + FROM + '; two challengers judged every one. Turn the\n' +
  'survivors into a single, ordered RFC change-set that the owner can act on, and that a later agent can apply\n' +
  'mechanically to produce v' + TO + '.\n\n' +
  'WRITE YOUR OUTPUT TO: ' + OUT + '\n' +
  'Use the Write tool. This is the only file you may create. Do not edit any other file in the repository.\n\n' +
  'DOCUMENT STRUCTURE:\n' +
  '  1. Summary - what v' + TO + ' changes and why, in a page. Written for the owner, who knows this spec intimately.\n' +
  '  2. An honest assessment of how close v' + FROM + ' is to the stated vision, pillar by pillar (C speed / Python\n' +
  '     readability / Rust safety / less annoying / C++ reuse / loved by people). Say where it falls short.\n' +
  '  3. The RFCs, numbered RFC-001 upward, ORDERED BY VALUE TO THE VISION, not by spec part. Each RFC:\n' +
  '       - Title, severity, status (proposed / needs-owner-decision), and the spec locations it touches\n' +
  '       - Problem (with the concrete evidence that proves it)\n' +
  '       - Proposal\n' +
  '       - Exact spec delta: the normative text to insert or replace, with rule ids, in the spec\'s own voice\n' +
  '       - Cost and blast radius: implementation work, what breaks, which compiler crates are affected\n' +
  '       - Provenance: originating finding id(s) and what the challengers said, including any revision that\n' +
  '         was demanded and that you have applied\n' +
  '  4. A section "Requires an owner decision" in the style of Part XXII.1, for anything you must not decide.\n' +
  '  5. A section "Rejected, and why" - list the killed findings compactly with the reason. The owner should be\n' +
  '     able to see what was considered and dismissed, so this exercise does not get repeated.\n' +
  '  6. An application order: which RFCs to apply first, and which depend on which.\n\n' +
  'EDITORIAL STANDARDS:\n' +
  '  - Apply every "accept-with-changes" revision the challengers demanded. Say that you did.\n' +
  '  - Merge RFCs that are really one change. Split any that bundle unrelated changes.\n' +
  '  - Where two accepted proposals conflict (see cross_cutting), resolve it and justify the resolution.\n' +
  '  - Reconciliation findings (RECON-*) are the highest priority regardless of how interesting other RFCs are:\n' +
  '    the specification and the shipped compiler currently disagree, and nothing else can be trusted until\n' +
  '    they agree. Put them first, but keep them compact - they are mechanical.\n' +
  '  - Do not pad. An RFC that does not change what the owner does is noise.\n' +
  '  - Be direct about trade-offs. The owner explicitly wants this language to be LOVED; tell them plainly where\n' +
  '    the current design will cost them affection, even where every individual rule is defensible.\n\n' +
  '================ SURVIVING FINDINGS (' + survivors.length + ') ================\n' +
  JSON.stringify(survivors, null, 2) +
  '\n\n================ REJECTED / MERGED FINDINGS (' + killed.length + ') ================\n' +
  JSON.stringify(killed.map((k) => ({ id: k.finding.id, title: k.finding.title, severity: k.finding.severity, verdicts: k.verdicts })), null, 2) +
  '\n\n================ CROSS-CUTTING AND MISSED, FROM THE CHALLENGERS ================\n' +
  challenges.map((c, i) => '--- challenger ' + (i + 1) + ' ---\ncross_cutting: ' + (c.cross_cutting || '') + '\nmissed: ' + (c.missed || '')).join('\n\n') +
  '\n\n================ LENS VERDICTS ================\n' + lensVerdicts,
  { label: 'synthesise:rfc-v' + TO + '', phase: 'Synthesise', schema: SYNTH_SCHEMA }
)

return {
  lenses_returned: reports.length,
  findings_total: findings.length,
  survivors: survivors.length,
  rejected: killed.length,
  synthesis: synth,
}
