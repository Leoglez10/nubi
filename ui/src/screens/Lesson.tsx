// Práctica: one exercise per card, a footer that says right or "try again"
// with a hint, unlimited retries, and a result with stars and what to review. The engine owns the
// session; this screen keeps the student's picks and the shuffled display order in the store so
// leaving for the class and coming back keeps the exercise.
import { useEffect, useRef, useState, type KeyboardEvent, type RefObject } from 'react'
import { api, type Answer, type Content, type Exercise, type LessonView } from '../api'
import { Nubi, type Mood } from '../art'
import { classOf, currentClass } from '../nav'
import { useApp, type LessonDraft, type LessonState } from '../store'
import { stars } from './Home'
import './Lesson.css'

const EMPTY: LessonDraft = { choice: null, command: '', picked: [], pairs: [], left: null }

const TITLES: Record<Exercise['type'], string> = {
  choice: 'Elige la respuesta correcta',
  command: 'Escribe el comando',
  order: 'Ordena los pasos',
  match: 'Une las parejas',
}

/** Stars for a finished attempt, like `grading::stars`. */
const starsFor = (errors: number) => (errors === 0 ? 3 : errors <= 2 ? 2 : 1)
const plural = (n: number, one: string, many: string) => `${n} ${n === 1 ? one : many}`

function exerciseOf(content: Content, view: LessonView): Exercise | undefined {
  return content.nodes[view.id]?.exercises[view.session.index]
}

/**
 * The student's answer, or null while incomplete. Order maps display positions back to content
 * indices (the engine grades content indices); match pairs are already content indices.
 */
export function answerOf(ex: Exercise, draft: LessonDraft, display: number[]): Answer | null {
  switch (ex.type) {
    case 'choice':
      return draft.choice == null ? null : { type: 'choice', value: draft.choice }
    case 'command': {
      const t = draft.command.trim()
      return t ? { type: 'command', value: t } : null
    }
    case 'order':
      return draft.picked.length === ex.steps.length ? { type: 'order', value: draft.picked.map((d) => display[d]) } : null
    case 'match':
      return draft.pairs.length === ex.pairs.length ? { type: 'match', value: draft.pairs } : null
  }
}

/** A fresh display order for an order/match exercise; asked once per exercise. */
async function displayFor(content: Content, view: LessonView): Promise<number[]> {
  const ex = view.finished ? undefined : exerciseOf(content, view)
  if (ex?.type === 'order') return api.shuffled(ex.steps.length)
  if (ex?.type === 'match') return api.shuffled(ex.pairs.length)
  return []
}

export function Lesson() {
  const { state, set, go } = useApp()
  const { content } = state
  const id = state.stopId ?? currentClass(content, state.me?.progress)?.node.id ?? null
  const lesson = state.lesson?.view.id === id ? state.lesson : null
  const [error, setError] = useState<string | null>(null)
  const latest = useRef(state.lesson)
  latest.current = state.lesson

  const fail = (e: unknown) => setError(String(e))

  /** Moves to a new engine view: new exercise, so new display order and empty picks. */
  const show = async (view: LessonView) => {
    const display = await displayFor(content, view)
    set({ lesson: { view, display, draft: EMPTY, reviewing: false }, ...(view.saved ? { me: view.saved } : {}) })
  }

  const begin = (stop: string) => {
    setError(null)
    return api.lessonStart(stop).then(show).catch(fail)
  }

  // Resume the same unfinished practice (the engine still holds it); anything else starts anew.
  useEffect(() => {
    if (!id) return
    const kept = latest.current
    if (kept?.view.id === id && !kept.view.finished) return
    void begin(id)
    // Only a new stop restarts the practice; `begin` writes through the store.
  }, [id])

  // A new exercise or the result: hand focus to its heading.
  const heading = useRef<HTMLHeadingElement>(null)
  const step = lesson ? `${lesson.view.session.index}:${lesson.view.finished}` : ''
  const first = useRef(true)
  useEffect(() => {
    if (!step) return
    if (first.current) { first.current = false; return }
    heading.current?.focus({ preventScroll: false })
  }, [step])

  const node = id ? content.nodes[id] : undefined
  if (!id || !node) {
    return (
      <section className="page">
        <h1 id="page-title" tabIndex={-1}>Práctica</h1>
      </section>
    )
  }
  if (!lesson) {
    return (
      <section className="page lesson-page">
        <h1 id="page-title" tabIndex={-1} className="visually-hidden">Práctica</h1>
        {error ? <p role="alert" className="note error">{error}</p> : <p className="note">Cargando la práctica…</p>}
      </section>
    )
  }

  const update = (patch: Partial<LessonState>) => set({ lesson: { ...lesson, ...patch } })
  const props = { lesson, heading, error, update, fail }
  return lesson.view.finished
    ? <Result {...props} again={() => begin(id)} />
    : <Exercising {...props} onExplain={() => {
        const shown = classOf(content, id)
        if (shown) go('class', { stopId: shown.node.id, lesson: { ...lesson, reviewing: true } })
      }} />
}

interface Props {
  lesson: LessonState
  heading: RefObject<HTMLHeadingElement | null>
  error: string | null
  update: (patch: Partial<LessonState>) => void
  fail: (e: unknown) => void
}

function Exercising({ lesson, heading, error, update, fail, onExplain }: Props & { onExplain: () => void }) {
  const { state, set, reducedMotion } = useApp()
  const { view, draft, display } = lesson
  const ex = exerciseOf(state.content, view)
  const { index, total, feedback } = view.session
  const locked = feedback != null
  const answer = ex ? answerOf(ex, draft, display) : null
  const teaching = classOf(state.content, view.id) != null

  const setDraft = (patch: Partial<LessonDraft>) => update({ draft: { ...draft, ...patch } })
  const check = () => {
    if (answer && !locked) api.lessonCheck(answer).then((v) => update({ view: v }), fail)
  }
  const retry = () => api.lessonRetry().then((v) => update({ view: v, draft: EMPTY }), fail)
  const next = async () => {
    try {
      const v = await api.lessonNext()
      const d = await displayFor(state.content, v)
      set({ lesson: { view: v, display: d, draft: EMPTY, reviewing: false }, ...(v.saved ? { me: v.saved } : {}) })
    } catch (e) { fail(e) }
  }

  const n = Math.min(index + 1, total)
  const percent = Math.round(view.progress * 100)
  const mood: Mood = feedback == null ? 'neutral' : feedback.type === 'right' ? 'happy' : 'worried'

  return (
    <div className="page lesson-page" data-reduced-motion={reducedMotion || undefined}>
      <article className="card lesson-card" aria-labelledby="page-title">
        <header className="lesson-top">
          <span className="lesson-count">{`Pregunta ${n} de ${total}`}</span>
          <span className="lesson-bar" role="progressbar" aria-label="Progreso de la práctica"
            aria-valuemin={0} aria-valuemax={100} aria-valuenow={percent}>
            <span style={{ inlineSize: `${percent}%` }} />
          </span>
          {teaching && <button type="button" className="btn btn-sm" onClick={onExplain}>Ver explicación</button>}
        </header>

        {ex && (
          <div className="lesson-body">
            <p className="eyebrow">{TITLES[ex.type].toUpperCase()}</p>
            <h1 id="page-title" ref={heading} tabIndex={-1} className="lesson-prompt">{ex.prompt}</h1>
            {ex.type === 'choice' && <Choice options={ex.options} draft={draft} locked={locked} onPick={(i) => setDraft({ choice: i })} />}
            {ex.type === 'command' && (
              <Command value={draft.command} locked={locked} onChange={(command) => setDraft({ command })}
                onEnter={() => (feedback?.type === 'right' ? void next() : feedback == null && check())} />
            )}
            {ex.type === 'order' && <Order steps={ex.steps} display={display} picked={draft.picked} locked={locked}
              onChange={(picked) => setDraft({ picked })} />}
            {ex.type === 'match' && <Match pairs={ex.pairs} display={display} draft={draft} locked={locked} onChange={setDraft} />}
          </div>
        )}

        <footer className={`lesson-footer ${feedback?.type ?? 'idle'}`}>
          <span className="lesson-nubi" data-mood={mood}><Nubi size={44} mood={mood} /></span>
          <div className="lesson-message">
            {feedback == null && (
              <p className="note">{answer ? '¿Lista tu respuesta? Compruébala.' : 'Elige una respuesta para comprobar.'}</p>
            )}
            <div role="status" className="lesson-feedback">
              {feedback?.type === 'right' && <p className="lesson-right">¡Correcto!</p>}
              {feedback?.type === 'wrong' && (
                <>
                  <p className="lesson-wrong">Casi. Inténtalo otra vez, sin límite.</p>
                  <p className="lesson-hint">{`Pista: ${feedback.hint}`}</p>
                </>
              )}
            </div>
            {error && <p role="alert" className="note error">{error}</p>}
          </div>
          {feedback == null && <button type="button" className="btn btn-primary" disabled={!answer} onClick={check}>Comprobar</button>}
          {feedback?.type === 'right' && <button type="button" className="btn btn-good" onClick={() => void next()}>Continuar</button>}
          {feedback?.type === 'wrong' && <button type="button" className="btn btn-danger" onClick={() => void retry()}>Intentar de nuevo</button>}
        </footer>
      </article>
    </div>
  )
}

function Choice({ options, draft, locked, onPick }: { options: string[]; draft: LessonDraft; locked: boolean; onPick: (i: number) => void }) {
  return (
    <fieldset className="lesson-choices" disabled={locked}>
      <legend className="visually-hidden">Opciones</legend>
      {options.map((o, i) => (
        <label key={i} className="tile choice">
          <input type="radio" name="choice" className="visually-hidden" checked={draft.choice === i} onChange={() => onPick(i)} />
          <span className="choice-num" aria-hidden="true">{i + 1}</span>
          <span className="choice-text">{o}</span>
        </label>
      ))}
    </fieldset>
  )
}

function Command({ value, locked, onChange, onEnter }: { value: string; locked: boolean; onChange: (v: string) => void; onEnter: () => void }) {
  const key = (e: KeyboardEvent<HTMLInputElement>) => {
    if (e.key === 'Enter') { e.preventDefault(); onEnter() }
  }
  return (
    <div className="lesson-command">
      <span className="prompt" aria-hidden="true">$</span>
      <input type="text" aria-label="Tu comando" placeholder="escribe el comando aquí" value={value} readOnly={locked}
        autoComplete="off" autoCapitalize="off" spellCheck={false} onChange={(e) => onChange(e.target.value)} onKeyDown={key} />
    </div>
  )
}

function Order({ steps, display, picked, locked, onChange }:
  { steps: string[]; display: number[]; picked: number[]; locked: boolean; onChange: (p: number[]) => void }) {
  return (
    <div className="lesson-order">
      <p className="note" id="order-help">Tu orden (toca un paso para quitarlo desde ahí):</p>
      <ol className="order-chosen" aria-labelledby="order-help">
        {picked.map((d, k) => (
          <li key={d}>
            <button type="button" className="tile on" disabled={locked} onClick={() => onChange(picked.slice(0, k))}>
              {`${k + 1}. ${steps[display[d]]}`}
            </button>
          </li>
        ))}
      </ol>
      <ul className="order-pool" aria-label="Pasos por colocar">
        {display.map((step, d) => picked.includes(d) ? null : (
          <li key={d}>
            <button type="button" className="tile" disabled={locked} onClick={() => onChange([...picked, d])}>{steps[step]}</button>
          </li>
        ))}
      </ul>
    </div>
  )
}

function Match({ pairs, display, draft, locked, onChange }:
  { pairs: [string, string][]; display: number[]; draft: LessonDraft; locked: boolean; onChange: (patch: Partial<LessonDraft>) => void }) {
  const pairOfLeft = (l: number) => draft.pairs.findIndex(([a]) => a === l)
  const pairOfRight = (r: number) => draft.pairs.findIndex(([, b]) => b === r)
  const pickLeft = (i: number) => onChange({ pairs: draft.pairs.filter(([a]) => a !== i), left: i })
  const pickRight = (r: number) => {
    const l = draft.left
    if (l == null) return
    onChange({ pairs: [...draft.pairs.filter(([a, b]) => a !== l && b !== r), [l, r]], left: null })
  }
  const tag = (n: number) => n >= 0 && (
    <>
      <span className="pair-num" aria-hidden="true">{n + 1}</span>
      <span className="visually-hidden">{` (pareja ${n + 1})`}</span>
    </>
  )
  return (
    <div className="lesson-match">
      <p className="note">Toca un concepto y luego su significado.</p>
      <div className="match-cols">
        <ul aria-label="Conceptos">
          {pairs.map(([concept], i) => {
            const n = pairOfLeft(i)
            return (
              <li key={i}>
                <button type="button" className={`tile${n >= 0 ? ` paired p${n % 5}` : ''}`} aria-pressed={draft.left === i}
                  disabled={locked} onClick={() => pickLeft(i)}>
                  <strong>{concept}</strong>{tag(n)}
                </button>
              </li>
            )
          })}
        </ul>
        <ul aria-label="Significados">
          {display.map((r, d) => {
            const n = pairOfRight(r)
            return (
              <li key={d}>
                <button type="button" className={`tile${n >= 0 ? ` paired p${n % 5}` : ''}`}
                  disabled={locked || draft.left == null} onClick={() => pickRight(r)}>
                  <span>{pairs[r][1]}</span>{tag(n)}
                </button>
              </li>
            )
          })}
        </ul>
      </div>
    </div>
  )
}

function Result({ lesson, heading, error, again }: Props & { again: () => void }) {
  const { state, go } = useApp()
  const { errors, mistakes } = lesson.view.session
  const n = starsFor(errors)
  const shown = classOf(state.content, lesson.view.id)
  return (
    <div className="page lesson-page">
      <article className="card lesson-card lesson-result" aria-labelledby="page-title">
        <Nubi size={120} mood={errors === 0 ? 'happy' : 'neutral'} />
        <h1 id="page-title" ref={heading} tabIndex={-1}>{errors === 0 ? '¡Lección perfecta!' : '¡Lección completada!'}</h1>
        <p className="lesson-stars" role="img" aria-label={`${n} de 3 estrellas`}>{stars(n)}</p>
        <p className="lesson-summary">
          {errors === 0 ? 'Cero errores: dominas esta lección.' : `Tuviste ${plural(errors, 'error', 'errores')}. 3 estrellas = cero errores.`}
        </p>
        {mistakes.length > 0 && (
          <section className="lesson-review" aria-labelledby="lesson-review-title">
            <h2 id="lesson-review-title">Para repasar</h2>
            <ul>{mistakes.map((m, i) => <li key={i}>{m}</li>)}</ul>
          </section>
        )}
        <div className="lesson-actions">
          {errors > 0
            ? <button type="button" className="btn btn-reward" onClick={again}>Perfeccionar</button>
            : <button type="button" className="btn" onClick={again}>Repetir</button>}
          {shown && <button type="button" className="btn btn-primary" onClick={() => go('lab', { stopId: shown.teaching.lab })}>Ir al laboratorio</button>}
          <button type="button" className="btn" onClick={() => go('home', { stopId: null })}>Volver al inicio</button>
        </div>
        {state.me && <p className="note">Tu resultado quedó guardado.</p>}
        {error && <p role="alert" className="note error">{error}</p>}
      </article>
    </div>
  )
}
