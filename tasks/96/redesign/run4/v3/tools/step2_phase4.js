export const meta = {
  name: 'ember-run4-step2-readability',
  description: 'Run 4 step 2, phase 4: the readability checker (duty 07) on the v3 rules, words and 84 answers, after the lead\'s message fixes',
  phases: [
    { title: 'Readability', detail: 'readability checker' },
  ],
}
const ROOT = '/home/user/ember/'
const V3 = ROOT + 'tasks/96/redesign/run4/v3/'
function header(n, name, section) {
  return `You are agent ${n} (${name}) of run 4, step 2, of the Ember memory-model rework. ` +
    `Read ${V3}inputs/brief.md first, all of it, then ${V3}inputs/JOB.md in full, then your duty: ` +
    `the section "${section}" of ${V3}inputs/duties.md. ` +
    `Scratch files go under ${ROOT}target/redesign4/v3-${n}-${name}/. When you are done, fill in the form exactly; ` +
    `finished is true only if you did your whole duty. `
}
const RETRY = ' NOTE: an earlier attempt at this duty did not finish. Check the files it may have changed and complete them rather than starting over.'
function halt(msg) { log('STOP: ' + msg); throw new Error('run 4 step 2 phase 4 stopped: ' + msg) }
async function run(n, name, prompt, opts) {
  let last = null
  for (let attempt = 1; attempt <= 2; attempt++) {
    const r = await agent(prompt + (attempt === 2 ? RETRY : ''), { ...opts, label: `${n} ${name}${attempt === 2 ? ' (retry)' : ''}` })
    if (r && r.finished) return r
    last = r
    log(`${n} ${name} did not finish (attempt ${attempt}): ${r ? r.reason : 'no result'}`)
  }
  halt(`${n} ${name} did not finish twice: ${last ? last.reason : 'no result'}`)
}
const S = { type: 'string' }, B = { type: 'boolean' }, I = { type: 'integer' }
const arr = items => ({ type: 'array', items })
const obj = (props, req) => ({ type: 'object', properties: props, required: req || Object.keys(props) })

const READ_SCHEMA = obj({
  finished: B, reason: S, report_path: S,
  blocks_checked: I,
  failures: arr(obj({ id: S, block: S, test: S, why: S })),
  verdicts: arr(obj({ number: I, applied_everywhere: B, where_not: S })),
  v2_failures: arr(obj({ id: { type: 'string', description: 'F-01 .. F-11 or F-13' }, fixed: B, where_not: S })),
  agreed_rows: arr(obj({ row: I, exact: B })),
  for_owner: arr(S),
})

phase('Readability')
const r = await run('07', 'readability', header('07', 'readability', '07 Readability checker'),
  { phase: 'Readability', schema: READ_SCHEMA, effort: 'high' })
const WANT = Array.from({ length: 52 }, (_, i) => i + 1).filter(n => n !== 25)
const got = new Set(r.verdicts.map(v => v.number))
const missing = WANT.filter(n => !got.has(n))
if (missing.length) log('WARNING the checker did not report verdicts ' + missing.join(' '))
const V2F = ['F-01', 'F-02', 'F-03', 'F-04', 'F-05', 'F-06', 'F-07', 'F-08', 'F-09', 'F-10', 'F-11', 'F-13']
const gotF = new Set(r.v2_failures.map(f => f.id))
const missF = V2F.filter(f => !gotF.has(f))
if (missF.length) log('WARNING the checker did not report ' + missF.join(' '))
log(`readability: ${r.blocks_checked} blocks, ${r.failures.length} failures; verdicts not applied everywhere: ${r.verdicts.filter(v => !v.applied_everywhere).map(v => v.number).join(' ') || 'none'}; v2 failures not fixed: ${r.v2_failures.filter(f => !f.fixed).map(f => f.id).join(' ') || 'none'}`)
return r
