// Clase: the explanation as an article with drawings, and beside it the
// class's parts and a tip from Nubi.
import { useEffect, useState } from 'react'
import type { Diagram, Teaching } from '../api'
import { Nubi } from '../art'
import { classOf, completed, currentClass } from '../nav'
import { useApp } from '../store'
import { stars } from './Home'
import './Class.css'

export function Class() {
  const { state, set, go } = useApp()
  const shown = classOf(state.content, state.stopId) ?? currentClass(state.content, state.me?.progress)
  // Opened from a practice or a lab ("Ver explicación", "Repasar la clase"): offer only the way
  // back to it (`class_return`). Read once on arrival, so a later visit from Inicio or the
  // sidebar is a normal class again.
  const [fromLesson] = useState(() => state.lesson?.reviewing === true)
  const [fromLab] = useState(() => state.lab?.reviewing ?? null)
  useEffect(() => {
    if (state.lesson?.reviewing) set({ lesson: { ...state.lesson, reviewing: false } })
  }, [state.lesson, set])
  useEffect(() => {
    if (state.lab?.reviewing) set({ lab: { ...state.lab, reviewing: null } })
  }, [state.lab, set])
  if (!shown) {
    return (
      <section className="page">
        <h1 id="page-title" tabIndex={-1}>Clase</h1>
      </section>
    )
  }
  const { node, teaching: t } = shown
  const progress = state.me?.progress
  const labReady = completed(progress, node.id)

  return (
    <div className="page class-page">
      <article className="card class-article" aria-labelledby="page-title">
        <header className="class-head">
          <p className="eyebrow">{`CLASE ${t.number} · ${fromLab === 'review' ? 'REPASO' : 'EXPLICACIÓN'}`}</p>
          <h1 id="page-title" tabIndex={-1}>{t.title}</h1>
        </header>
        <p className="class-explanation">{t.explanation}</p>
        {t.diagram
          ? <DiagramView diagram={t.diagram} caption={t.visual} />
          : <p className="class-visual">{t.visual}</p>}

        <h2>Ejemplo resuelto</h2>
        <p className="class-example">{t.example}</p>
        {t.diagram && <ExampleView diagram={t.diagram} />}

        <ReviewCard review={t.review} />

        <div className="class-actions">
          {fromLesson && state.lesson?.view.id === node.id ? (
            <button type="button" className="btn btn-primary" onClick={() => go('lesson', { stopId: node.id })}>
              Volver a la práctica
            </button>
          ) : fromLab && state.lab?.view.id === t.lab ? (
            <button type="button" className="btn btn-primary"
              onClick={() => go('lab', { stopId: t.lab, lab: { ...state.lab!, reviewing: null, resume: true } })}>
              Volver al laboratorio
            </button>
          ) : <>
            <button type="button" className="btn btn-primary" onClick={() => go('lesson', { stopId: node.id, lesson: null })}>
              Empezar práctica
            </button>
            {labReady
              ? <button type="button" className="btn btn-primary" onClick={() => go('lab', { stopId: t.lab })}>Ir al laboratorio</button>
              : <button type="button" className="btn" disabled>Laboratorio: termina la práctica</button>}
            <button type="button" className="btn" onClick={() => go('home', { stopId: null })}>Volver al inicio</button>
          </>}
        </div>
      </article>
      <Aside id={node.id} teaching={t} />
    </div>
  )
}

/** The class's parts, where each stands, and Nubi's tip. */
function Aside({ id, teaching }: { id: string; teaching: Teaching }) {
  const progress = useApp().state.me?.progress
  const best = (key: string) => progress?.nodes[key]?.best_stars
  const practice = best(id)
  const lab = best(teaching.lab)
  const practiced = completed(progress, id)
  const labDone = completed(progress, teaching.lab)
  const steps: { label: string; meta: string; done: boolean; now?: boolean }[] = [
    { label: 'Explicación', meta: '', done: true, now: true },
    { label: 'Ejemplo', meta: '', done: practiced },
    { label: 'Práctica', meta: practice != null ? stars(practice) : '0 / 5', done: practiced },
    { label: 'Laboratorio', meta: lab != null ? stars(lab) : practiced ? 'Listo' : 'Bloqueado', done: labDone },
    { label: 'Repaso', meta: '', done: labDone },
  ]
  return (
    <aside className="class-aside" aria-label="En esta clase">
      <section className="card class-steps">
        <h2>En esta clase</h2>
        <ol>
          {steps.map((s, i) => {
            const state = s.now ? 'now' : s.done ? 'done' : 'todo'
            return (
              <li key={s.label} className={`class-step ${state}`} aria-current={s.now ? 'step' : undefined}>
                <span className="step-dot" aria-hidden="true">{state === 'done' ? '✓' : i + 1}</span>
                <span className="step-label">{s.label}</span>
                {s.meta && <span className="step-meta">{s.meta}</span>}
                {state === 'done' && <span className="visually-hidden">(hecho)</span>}
              </li>
            )
          })}
        </ol>
        <div className="class-nubi">
          <Nubi size={40} />
          <div>
            <p className="class-nubi-title">Tip de Nubi</p>
            <p>{teaching.visual}</p>
          </div>
        </div>
      </section>
    </aside>
  )
}

/** A drawn idea plus its caption, so the explanation is not only words. */
function DiagramView({ diagram, caption }: { diagram: Diagram; caption: string }) {
  return (
    <figure className="class-diagram" data-diagram={diagram}>
      <div className="class-diagram-art">{diagram === 'bucket_objects' ? <BucketObjects /> : <KeyPrefixes />}</div>
      <figcaption>{caption}</figcaption>
    </figure>
  )
}

/** One bucket with three objects inside: the bucket holds keys, not folders. */
function BucketObjects() {
  return (
    <div className="diagram-bucket">
      <p className="diagram-bucket-name"><span className="s3-chip" aria-hidden="true">S3</span>Bucket: mi-bucket</p>
      <ul className="diagram-objects">
        {['foto.jpg', 'informe.txt', 'video.mp4'].map((name) => (
          <li key={name}>
            <code>{name}</code>
            <span>{`clave: ${name}`}</span>
          </li>
        ))}
      </ul>
    </div>
  )
}

/** A flat key read left to right: the prefixes only look like folders. */
function KeyPrefixes() {
  const parts = [['BUCKET', 'archivo-nubi', 'bucket'], ['PREFIJO', 'informes/', 'prefix'], ['NOMBRE', 'enero.txt', 'name']]
  return (
    <ol className="diagram-key" aria-label="Partes de una clave">
      {parts.map(([role, part, tone]) => (
        <li key={role} className={tone}>
          <span>{role}</span>
          <code>{part}</code>
        </li>
      ))}
    </ol>
  )
}

/** The worked example drawn as what you type and what you get. */
function ExampleView({ diagram }: { diagram: Diagram }) {
  const [command, output, looks, is] = diagram === 'bucket_objects'
    ? ['aws s3api create-bucket --bucket mi-bucket', '{ "Location": "/mi-bucket" }',
      ['mi-bucket/', '  (vacío)'], ['Bucket creado, listo para objetos']]
    : ['aws s3 cp enero.txt s3://archivo-nubi/informes/enero.txt', 'upload: ./enero.txt to s3://archivo-nubi/informes/enero.txt',
      ['informes/', '  ├ enero.txt', '  └ febrero.txt'], ['informes/enero.txt', 'informes/febrero.txt']]
  return (
    <div className="class-example-visual">
      <pre className="example-term"><code><span className="prompt" aria-hidden="true">$ </span>{command}{'\n'}<span className="out">{output}</span></code></pre>
      <div className="example-sides">
        <Side title="Lo que muestra la consola" lines={looks as string[]} />
        <Side title="Lo que S3 guarda" lines={is as string[]} />
      </div>
    </div>
  )
}

function Side({ title, lines }: { title: string; lines: string[] }) {
  return (
    <section className="example-side">
      <h3>{title}</h3>
      <pre>{lines.join('\n')}</pre>
    </section>
  )
}

/** The takeaway with a check, easy to spot when revisiting. */
function ReviewCard({ review }: { review: string }) {
  return (
    <section className="class-review" aria-labelledby="class-review-title">
      <span className="review-check" aria-hidden="true">✓</span>
      <div>
        <h2 id="class-review-title">Para recordar</h2>
        <p>{review}</p>
      </div>
    </section>
  )
}
