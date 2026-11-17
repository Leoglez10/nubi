// Laboratorio: objectives on one side, the integrated terminal on the other,
// and the result with stars, what went wrong and "Perfeccionar". The engine owns the session;
// this screen keeps the typed line and the running command in the store so leaving for the class
// and coming back keeps the attempt.
import { useEffect, useRef, useState, type KeyboardEvent, type RefObject } from 'react'
import { api, type LabView, type LineKind } from '../api'
import { classOf, currentClass } from '../nav'
import { useApp, type LabState } from '../store'
import { stars } from './Home'
import './Lab.css'

/** Stars for a finished attempt, like `grading::stars`. */
const starsFor = (errors: number) => (errors === 0 ? 3 : errors <= 2 ? 2 : 1)
const plural = (n: number, one: string, many: string) => `${n} ${n === 1 ? one : many}`

const PROMPT = 'nubi@labcloud $'
const KIND_LABEL: Record<LineKind, string> = { cmd: 'comando', out: 'salida', err: 'error', ok: 'correcto', info: 'info' }

export function Lab() {
  const { state, set, go } = useApp()
  const { content } = state
  const id = state.stopId ?? currentClass(content, state.me?.progress)?.teaching.lab ?? null
  const lab = state.lab?.view.id === id ? state.lab : null
  const [error, setError] = useState<string | null>(null)
  const latest = useRef(state.lab)
  latest.current = state.lab

  const fresh = (view: LabView): LabState => ({ view, input: '', running: null, reviewing: null, resume: false })
  const begin = (stop: string) => {
    setError(null)
    return api.labStart(stop).then((view) => set({ lab: fresh(view) }), (e) => setError(String(e)))
  }

  // Resume the same unfinished lab (the engine still holds it) or the attempt left for Clase;
  // anything else starts anew with fresh resources (`open_stop`).
  useEffect(() => {
    if (!id) return
    const kept = latest.current
    if (kept?.view.id === id && (!kept.view.complete || kept.resume)) {
      if (kept.resume) set({ lab: { ...kept, resume: false } })
      return
    }
    void begin(id)
    // Only a new stop restarts the lab; `begin` writes through the store.
  }, [id])

  const node = id ? content.nodes[id] : undefined
  if (!id || !node) {
    return (
      <section className="page">
        <h1 id="page-title" tabIndex={-1}>Laboratorio</h1>
      </section>
    )
  }
  if (!lab) {
    return (
      <section className="page lab-page">
        <p className="eyebrow">LABORATORIO</p>
        <h1 id="page-title" tabIndex={-1}>{node.title}</h1>
        {error
          ? <Trouble error={error} retry={() => void begin(id)} />
          : <p className="note" role="status">Preparando el laboratorio…</p>}
      </section>
    )
  }

  const toClass = (reviewing: 'explain' | 'review') => {
    const shown = classOf(content, id)
    if (shown) go('class', { stopId: shown.node.id, lab: { ...lab, reviewing } })
  }
  return <Playing lab={lab} title={node.title} error={error} setError={setError}
    again={() => void begin(id)} toClass={classOf(content, id) ? toClass : undefined} />
}

/** The engine could not start or run: say why and offer the way to fix the environment. */
function Trouble({ error, retry }: { error: string; retry?: () => void }) {
  const { go } = useApp()
  return (
    <div className="lab-trouble" role="alert">
      <p>{error}</p>
      <div className="lab-trouble-actions">
        {retry && <button type="button" className="btn btn-sm" onClick={retry}>Reintentar</button>}
        <button type="button" className="btn btn-sm btn-primary" onClick={() => go('setup')}>Revisar el entorno</button>
      </div>
    </div>
  )
}

interface PlayingProps {
  lab: LabState
  title: string
  error: string | null
  setError: (e: string | null) => void
  again: () => void
  toClass?: (reviewing: 'explain' | 'review') => void
}

function Playing({ lab, title, error, setError, again, toClass }: PlayingProps) {
  const { state, set, reducedMotion } = useApp()
  const { view } = lab
  const { lines, done, errors } = view.session
  const objectives = state.content.nodes[view.id]?.objectives ?? []
  const latest = useRef(lab)
  latest.current = lab
  const input = useRef<HTMLInputElement>(null)
  const scroll = useRef<HTMLDivElement>(null)
  const resultTitle = useRef<HTMLHeadingElement>(null)

  const update = (patch: Partial<LabState>) => set({ lab: { ...latest.current, ...patch } })

  const run = async () => {
    const line = lab.input
    if (lab.running != null || !line.trim()) return
    setError(null)
    update({ input: '', running: line })
    try {
      const next = await api.labSubmit(line)
      set({ lab: { ...latest.current, view: next, running: null }, ...(next.saved ? { me: next.saved } : {}) })
    } catch (e) {
      update({ running: null })
      setError(String(e))
    }
  }

  const history = async (older: boolean) => {
    try {
      const line = await api.labHistory(older)
      // Up with nothing older keeps the prompt; down past the newest clears it (`history_key`).
      if (line != null || !older) update({ input: line ?? '' })
    } catch (e) { setError(String(e)) }
  }

  const key = (e: KeyboardEvent<HTMLInputElement>) => {
    if (e.key === 'Enter') { e.preventDefault(); void run() }
    else if (e.key === 'ArrowUp') { e.preventDefault(); void history(true) }
    else if (e.key === 'ArrowDown') { e.preventDefault(); void history(false) }
  }

  // The newest line stays in view, and the prompt keeps the focus after each command.
  const busy = lab.running != null
  useEffect(() => {
    const el = scroll.current
    if (el) el.scrollTo?.({ top: el.scrollHeight, behavior: reducedMotion ? 'auto' : 'smooth' })
  }, [lines.length, busy, reducedMotion])
  const first = useRef(true)
  useEffect(() => {
    if (first.current) { first.current = false; return }
    if (!busy && !view.complete) input.current?.focus()
  }, [busy, view.complete])
  // Completing hands focus to the result, where the next step is.
  useEffect(() => {
    if (view.complete && lab.running == null) resultTitle.current?.focus()
  }, [view.complete])

  const nextHint = objectives[done.findIndex((d) => !d)]?.hint
  const suggestions: [string, string][] = [
    ...(nextHint && !view.complete ? [['siguiente paso', nextHint] as [string, string]] : []),
    ['pista', 'pista'],
    ['aws s3 ls', 'aws s3 ls'],
  ]

  return (
    <div className="page lab-page" data-reduced-motion={reducedMotion || undefined}>
      <header className="lab-head">
        <div className="lab-head-text">
          <p className="eyebrow">LABORATORIO</p>
          <h1 id="page-title" tabIndex={-1}>{title}</h1>
        </div>
        <span className={`lab-errors ${errors === 0 ? 'none' : 'some'}`} role="status">
          {errors === 0 ? 'Sin errores' : plural(errors, 'error', 'errores')}
        </span>
        {toClass && <button type="button" className="btn btn-sm" onClick={() => toClass('explain')}>Ver explicación</button>}
      </header>

      <div className="lab-body">
        <div className="lab-side">
          <section className="card lab-goals" aria-labelledby="lab-goals-title">
            <h2 id="lab-goals-title">Objetivos</h2>
            <ol>
              {objectives.map((o, i) => {
                const ok = done[i] ?? false
                return (
                  <li key={i} className={`lab-goal${ok ? ' done' : ''}`}>
                    <span className="goal-dot" aria-hidden="true">{ok ? '✓' : i + 1}</span>
                    <span className="goal-text">{o.text}</span>
                    {ok && <span className="visually-hidden">(hecho)</span>}
                  </li>
                )
              })}
            </ol>
          </section>
          {view.complete && <Result lab={lab} heading={resultTitle} again={again} toClass={toClass} />}
        </div>

        <section className="lab-term" aria-label="Terminal">
          <div className="term-bar">
            <span>Terminal</span>
            <span className="term-host">aws → 127.0.0.1:9297</span>
          </div>
          <div className="term-scroll" ref={scroll}>
            <ol className="term-lines" role="log" aria-label="Salida de la terminal">
              {lines.map((l, i) => (
                <li key={i} className={`term-line ${l.kind}`}>
                  <span className="visually-hidden">{`${KIND_LABEL[l.kind]}: `}</span>
                  {l.kind === 'cmd' && <span className="term-prompt" aria-hidden="true">{`${PROMPT} `}</span>}
                  {l.text}
                </li>
              ))}
              {busy && (
                <li className="term-line cmd">
                  <span className="visually-hidden">comando: </span>
                  <span className="term-prompt" aria-hidden="true">{`${PROMPT} `}</span>{lab.running}
                </li>
              )}
            </ol>
            {busy && <p className="term-line info term-busy" role="status">corriendo…</p>}
          </div>
          {error && <div className="term-error"><Trouble error={error} /></div>}
          <div className="term-foot">
            <div className="term-input">
              <span className="term-prompt" aria-hidden="true">{PROMPT}</span>
              <input ref={input} type="text" aria-label="Comando" value={lab.input}
                placeholder="escribe un comando y presiona Enter" readOnly={busy} aria-busy={busy}
                autoComplete="off" autoCapitalize="off" autoCorrect="off" spellCheck={false}
                onChange={(e) => update({ input: e.target.value })} onKeyDown={key} />
            </div>
            <div className="term-chips">
              <span className="term-chips-label">Sugerencias:</span>
              {suggestions.map(([label, value]) => (
                <button key={label} type="button" className="term-chip"
                  onClick={() => { update({ input: value }); input.current?.focus() }}>{label}</button>
              ))}
              <button type="button" className="btn btn-primary btn-sm term-run" disabled={busy} onClick={() => void run()}>
                Ejecutar
              </button>
            </div>
          </div>
        </section>
      </div>
    </div>
  )
}

function Result({ lab, heading, again, toClass }: {
  lab: LabState
  heading: RefObject<HTMLHeadingElement | null>
  again: () => void
  toClass?: (reviewing: 'explain' | 'review') => void
}) {
  const { state, go } = useApp()
  const { errors, mistakes } = lab.view.session
  const n = starsFor(errors)
  const best = state.me?.progress.nodes[lab.view.id]
  return (
    <section className="lab-result" aria-labelledby="lab-result-title">
      <h2 id="lab-result-title" ref={heading} tabIndex={-1}>{errors === 0 ? '¡Misión perfecta!' : '¡Misión completada!'}</h2>
      <p className="lab-stars" role="img" aria-label={`${n} de 3 estrellas`}>{stars(n)}</p>
      <p>{errors === 0
        ? 'Cero errores. Dominas este lab.'
        : `Tuviste ${plural(errors, 'error', 'errores')}. Repítelo desde cero para lograr la perfección.`}</p>
      {best && <p className="note">{`Tu mejor marca: ${stars(best.best_stars)} · ${plural(best.best_errors, 'error', 'errores')}`}</p>}
      {mistakes.length > 0 && (
        <ul className="lab-mistakes" aria-label="Lo que falló">
          {mistakes.slice(0, 3).map((m, i) => (
            <li key={i}>
              <code>{m.command}</code>
              <span>{m.error}</span>
            </li>
          ))}
        </ul>
      )}
      <div className="lab-result-actions">
        {errors > 0
          ? <button type="button" className="btn btn-reward" onClick={again}>Perfeccionar</button>
          : <button type="button" className="btn btn-primary" onClick={again}>Repetir lab</button>}
        {toClass && <button type="button" className="btn" onClick={() => toClass('review')}>Repasar la clase</button>}
        <button type="button" className="btn" onClick={() => go('home', { stopId: null })}>Volver al inicio</button>
      </div>
      {state.me && <p className="note">Tu resultado quedó guardado.</p>}
    </section>
  )
}
