// Inicio: where the student stands, the class to continue and the module at a
// glance. Every number comes from saved progress.
import { classes, type Content, type ProgressView, type Stop } from '../api'
import { Nubi } from '../art'
import { completed, currentClass, type ClassRef } from '../nav'
import { useApp } from '../store'
import './Home.css'

/** ★★☆ for 2 of 3 stars. */
export const stars = (n: number) => '★'.repeat(n) + '☆'.repeat(Math.max(0, 3 - n))

/**
 * Where a stop stands on the map: the engine's `stops` when it lists it, else the same rule as
 * `grading::stop_status` (done with stars; open when it is first or the previous one is done).
 */
export function stopStatus(content: Content, me: ProgressView | null, id: string): Stop['type'] {
  const listed = me?.stops.find((s) => s.id === id)
  if (listed) return listed.status.type
  const progress = me?.progress
  if (completed(progress, id)) return 'done'
  const order = content.course.units.flatMap((u) => u.nodes)
  const i = order.indexOf(id)
  return i === 0 || (i > 0 && completed(progress, order[i - 1])) ? 'current' : 'locked'
}

export function Home() {
  const { state, go } = useApp()
  const { content, me } = state
  const progress = me?.progress
  const all = classes(content)
  const total = all.length
  const done = all.filter((c) => completed(progress, c.teaching.lab)).length
  const current = currentClass(content, progress)
  const allDone = total > 0 && done === total
  const tip = current && (allDone ? '¡Terminaste el módulo! Repasa para sacar 3 estrellas.' : `Hoy toca ${current.teaching.title.toLowerCase()}.`)

  const stops = content.course.units.reduce((n, u) => n + u.nodes.length, 0)
  const starCount = Object.values(progress?.nodes ?? {}).reduce((n, r) => n + r.best_stars, 0)
  const practiced = all.filter((c) => completed(progress, c.node.id)).length
  const kpis: [string, string, string][] = [
    ['Clases completadas', `${done} / ${total}`, 'Módulo de fundamentos'],
    ['Estrellas', String(starCount), `de ${stops * 3} posibles`],
    ['Prácticas', String(practiced), `de ${total}`],
    ['Nivel', String(me?.level.number ?? 1), me?.level.name ?? ''],
  ]

  return (
    <section className="page home">
      <header className="home-greeting">
        <div className="home-hello">
          <h1 id="page-title" tabIndex={-1}>{me ? `Hola, ${me.profile.name}` : 'Hola'}</h1>
          <p>{`Llevas ${done} de ${total} clases del módulo de fundamentos.`}</p>
        </div>
        {tip && (
          <aside className="home-nubi card" aria-label="Nubi">
            <Nubi size={56} mood={allDone ? 'happy' : 'neutral'} />
            <div>
              <p className="home-nubi-name">Nubi</p>
              <p className="home-nubi-tip">{tip}</p>
            </div>
          </aside>
        )}
      </header>

      <ul className="home-kpis" aria-label="Tu progreso">
        {kpis.map(([label, value, sub]) => (
          <li key={label} className="card kpi">
            <span className="kpi-label">{label}</span>
            <strong className="kpi-value">{value}</strong>
            <span className="kpi-sub">{sub}</span>
          </li>
        ))}
      </ul>

      {current && <ContinueCard shown={current} allDone={allDone} onOpen={() => go('class', { stopId: current.node.id })} />}

      <ModuleTable />
    </section>
  )
}

function ContinueCard({ shown, allDone, onOpen }: { shown: ClassRef; allDone: boolean; onOpen: () => void }) {
  const progress = useApp().state.me?.progress
  const finished = [shown.node.id, shown.teaching.lab].filter((id) => completed(progress, id)).length
  const t = shown.teaching
  return (
    <section className="card home-continue" aria-labelledby="home-continue-title">
      <p className="eyebrow">{allDone ? 'REPASA TU ÚLTIMA CLASE' : 'CONTINÚA DONDE LO DEJASTE'}</p>
      <div className="home-continue-body">
        <div className="home-continue-text">
          <h2 id="home-continue-title">{`Clase ${t.number} · ${t.title}`}</h2>
          <p>{t.explanation}</p>
          <div className="home-continue-progress">
            <span className="bar" aria-hidden="true"><span style={{ inlineSize: `${(finished / 2) * 100}%` }} /></span>
            <span className="note">{`${finished} de 2 partes`}</span>
          </div>
        </div>
        <button type="button" className="btn btn-primary" onClick={onOpen}>
          {finished === 0 ? 'Empezar clase' : 'Continuar clase'}
        </button>
      </div>
    </section>
  )
}

/** One row per class with its two parts, where it stands and how to open it. */
function ModuleTable() {
  const { state, go } = useApp()
  const { content, me } = state
  const progress = me?.progress
  const all = classes(content)
  const best = (id: string) => progress?.nodes[id]?.best_stars ?? 0

  return (
    <section className="card home-module" aria-labelledby="home-module-title">
      <header className="home-module-head">
        <h2 id="home-module-title">Módulo 1 · Fundamentos</h2>
        <span className="note">{`${all.length} clases`}</span>
      </header>
      <table className="module-table" aria-labelledby="home-module-title">
        <thead>
          <tr>
            <th scope="col">Clase</th>
            <th scope="col">Estado</th>
            <th scope="col"><span className="visually-hidden">Acciones</span></th>
          </tr>
        </thead>
        <tbody>
          {all.map(({ node, teaching: t }) => {
            const locked = stopStatus(content, me, node.id) === 'locked'
            const labDone = completed(progress, t.lab)
            const practiced = completed(progress, node.id)
            const [badge, tone] = labDone ? ['Completada', 'done'] : locked ? ['Bloqueada', 'locked'] : ['En curso', 'current']
            const detail = locked
              ? 'Disponible al terminar la clase anterior'
              : `Práctica ${stars(best(node.id))} · Laboratorio ${stars(best(t.lab))}`
            return (
              <tr key={node.id} className={`module-row ${tone}`}>
                <th scope="row">
                  <span className="module-class">
                    <span className="module-num" aria-hidden="true">{t.number}</span>
                    <span className="module-name">
                      <strong>{`Clase ${t.number}: ${t.title}`}</strong>
                      <span className="module-detail">{detail}</span>
                    </span>
                  </span>
                </th>
                <td><span className={`badge ${tone}`}>{badge}</span></td>
                <td>
                  <span className="module-actions">
                    <button type="button" className="btn btn-sm" disabled={locked}
                      aria-label={`Abrir la clase ${t.number}`} onClick={() => go('class', { stopId: node.id })}>
                      Clase
                    </button>
                    <button type="button" className="btn btn-sm" disabled={locked}
                      aria-label={`Práctica de la clase ${t.number}`} onClick={() => go('lesson', { stopId: node.id })}>
                      Práctica
                    </button>
                    <button type="button" className="btn btn-sm" disabled={locked || !practiced}
                      aria-label={`Laboratorio de la clase ${t.number}`}
                      title={practiced ? undefined : 'Laboratorio: termina la práctica'}
                      onClick={() => go('lab', { stopId: t.lab })}>
                      Laboratorio
                    </button>
                  </span>
                </td>
              </tr>
            )
          })}
        </tbody>
      </table>
    </section>
  )
}
