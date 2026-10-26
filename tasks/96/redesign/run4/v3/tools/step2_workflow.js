export const meta = {
  name: 'ember-run4-step2-update',
  description: 'Run 4 step 2: bring the Ember memory-model rules and the 84 answers up to date with the owner\'s verdicts 21-34 (rule writer, then 5 area updaters at once)',
  phases: [
    { title: 'Rules', detail: 'rule writer' },
    { title: 'Areas', detail: '5 area updaters, all at once' },
  ],
}
const ROOT = 'C:\\Users\\ism19\\Code\\ember\\'
const V3 = ROOT + 'tasks\\96\\redesign\\run4\\v3\\'
function header(n, name, section) {
  return `You are agent ${n} (${name}) of run 4, step 2, of the Ember memory-model rework. ` +
    `Read ${V3}inputs\\brief.md first, all of it, then ${V3}inputs\\JOB.md in full, then your duty: ` +
    `the section "${section}" of ${V3}inputs\\duties.md. ` +
    `Scratch files go under ${ROOT}target\\redesign4\\v3-${n}-${name}\\. When you are done, fill in the form exactly; ` +
    `finished is true only if you did your whole duty. `
}
const RETRY = ' NOTE: an earlier attempt at this duty did not finish. Check the files it may have changed and complete them rather than starting over.'
function halt(msg) { log('STOP: ' + msg); throw new Error('run 4 step 2 stopped: ' + msg) }
async function run(n, name, prompt, opts) {
  let last = null
  for (let attempt = 1; attempt <= 2; attempt++) {
    const r = await agent(prompt + (attempt === 2 ? RETRY : ''), { ...opts, label: `${n} ${name}${attempt === 2 ? ' (retry)' : ''}` })
    if (r && r.on_mains === false) halt(`${n} ${name} reports the laptop is not on mains power`)
    if (r && r.finished) return r
    last = r
    log(`${n} ${name} did not finish (attempt ${attempt}): ${r ? r.reason : 'no result'}`)
  }
  halt(`${n} ${name} did not finish twice: ${last ? last.reason : 'no result'}`)
}
const S = { type: 'string' }, B = { type: 'boolean' }, I = { type: 'integer' }
const arr = items => ({ type: 'array', items })
const obj = (props, req) => ({ type: 'object', properties: props, required: req || Object.keys(props) })
const BASE = { finished: B, reason: S, on_mains: B }

const RULE_SCHEMA = obj({
  ...BASE, rules_path: S, vocabulary_path: S, changes_path: S,
  decisions: arr(obj({ number: I, applied: B, where: S })),
  retired_codes: arr(S), new_codes: arr(S),
  messages_left_for_lead: arr(obj({ code: S, why: S })),
  for_owner: arr(S),
})
const AREA_SCHEMA = obj({
  ...BASE, report_path: S,
  answer_ids: arr({ type: 'string', description: 'like AN-01' }),
  phase0_lines_checked: B,
  meaning_changed: arr(obj({ id: S, verdict: S, how: S })),
  words_needed: arr(obj({ word: S, why: S })),
  agreed_rows: arr(obj({ row: I, unchanged: { type: 'boolean', description: 'true when the row is exactly as agreed, in the verdicts\' spellings (row 6: fn() changes score:)' } })),
  messages_flagged: arr(obj({ answer: S, message: S, why: S })),
  for_owner: arr(S),
})

phase('Rules')
const rules = await run('01', 'rule-writer', header('01', 'rule-writer', '01 Rule writer'),
  { phase: 'Rules', schema: RULE_SCHEMA, effort: 'xhigh' })
const WANT = [21, 22, 23, 24, 26, 27, 28, 29, 30, 31, 32, 33, 34]
const nums = new Set(rules.decisions.filter(d => d.applied).map(d => d.number))
const missing = WANT.filter(n => !nums.has(n))
if (missing.length) halt('the rule writer did not apply verdicts ' + missing.join(' '))
log(`rules: all ${WANT.length} verdicts applied; retired codes ${rules.retired_codes.length}; new codes ${rules.new_codes.length}; messages left for the lead ${rules.messages_left_for_lead.length}`)

phase('Areas')
const range = (a, b) => Array.from({ length: b - a + 1 }, (_, i) => 'AN-' + String(a + i).padStart(2, '0'))
const AREAS = [
  ['02', 'area-A', 'A', range(1, 16), 'high'], ['03', 'area-B', 'B', range(17, 33), 'high'], ['04', 'area-C', 'C', range(34, 52), 'high'],
  ['05', 'area-D', 'D', range(53, 70), 'high'], ['06', 'area-E', 'E', range(71, 84), 'xhigh'],
]
const norm = id => { const m = String(id).match(/(\d+)/); return m ? 'AN-' + m[1].padStart(2, '0') : String(id) }
const areas = await parallel(AREAS.map(([n, name, letter, ids, effort]) => async () => {
  const prompt = header(n, name, '02-06 Area updaters A-E') + `You are area ${letter}; your answers are ${ids[0]} to ${ids[ids.length - 1]}, in ${V3}area-${letter}.md.`
  let r = await run(n, name, prompt, { phase: 'Areas', schema: AREA_SCHEMA, effort })
  let got = new Set(r.answer_ids.map(norm))
  let miss = ids.filter(id => !got.has(id))
  if (miss.length) {
    log(`${n} ${name} did not report ${miss.join(' ')}; asking once more`)
    r = await run(n, name + '-complete', prompt + ` These answers were not brought up to date; do them in V3/area-${letter}.md: ${miss.join(' ')}.`,
      { phase: 'Areas', schema: AREA_SCHEMA, effort })
    got = new Set([...got, ...r.answer_ids.map(norm)])
    miss = ids.filter(id => !got.has(id))
    if (miss.length) halt(`area ${letter} still misses ${miss.join(' ')}`)
  }
  return { letter, r }
}))
if (areas.some(a => !a)) halt('an area updater failed')
const changedRows = areas.flatMap(a => a.r.agreed_rows.filter(x => !x.unchanged).map(x => `${a.letter}: row ${x.row}`))
if (changedRows.length) log('WARNING agreed rows reported changed: ' + changedRows.join(', '))
log('areas: ' + areas.map(a => `${a.letter} ${a.r.answer_ids.length} answers, ${a.r.meaning_changed.length} changed, ${a.r.words_needed.length} words needed, ${a.r.messages_flagged.length} messages flagged`).join('; '))

return {
  rules: { retired_codes: rules.retired_codes, new_codes: rules.new_codes, messages_left_for_lead: rules.messages_left_for_lead, for_owner: rules.for_owner, decisions: rules.decisions },
  areas: areas.map(a => ({ area: a.letter, answers: a.r.answer_ids.length, phase0_lines_checked: a.r.phase0_lines_checked, meaning_changed: a.r.meaning_changed, words_needed: a.r.words_needed, agreed_rows_changed: a.r.agreed_rows.filter(x => !x.unchanged), messages_flagged: a.r.messages_flagged, for_owner: a.r.for_owner })),
}
