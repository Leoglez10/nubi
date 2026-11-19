import { useId, useLayoutEffect, useRef, useState, type ReactNode } from 'react'
import type { Teaching } from '../api'
import { Nubi } from '../art'
import { useApp } from '../store'

type Scene = {
  label: string
  title: string
  body: ReactNode
  action?: { label: string; run?: () => void }
  actionBelow?: boolean
}

/** A class supplies the scenes; navigation, progress and focus work the same for each class. */
function SceneDeck({ scenes }: { scenes: readonly [Scene, ...Scene[]] }) {
  const { reducedMotion } = useApp()
  const [index, setIndex] = useState(0)
  const heading = useRef<HTMLHeadingElement>(null)
  const previous = useRef(index)
  const titleId = useId()
  const scene = scenes[index]

  useLayoutEffect(() => {
    if (previous.current !== index) heading.current?.focus()
    previous.current = index
  }, [index])

  const action = scene.action && (
    <button type="button" className={`btn btn-primary scene-primary${scene.action.run ? ' scene-primary-interactive' : ''}`}
      onClick={scene.action.run ?? (() => setIndex(index + 1))}>
      {scene.action.label}
    </button>
  )

  return (
    <section className="scene-deck" aria-labelledby={titleId} data-motion={reducedMotion ? 'reduced' : 'full'} data-has-action={Boolean(scene.action)}>
      <header className="scene-header">
        <p className="eyebrow">Escena {index + 1} de {scenes.length}</p>
        <ol className="scene-progress" aria-label="Escenas de la clase">
          {scenes.map(({ label }, i) => (
            <li key={label} aria-current={i === index ? 'step' : undefined}><span aria-hidden="true">{i + 1}</span> {label}</li>
          ))}
        </ol>
        <h2 id={titleId} ref={heading} tabIndex={-1}>{scene.title}</h2>
      </header>
      {!scene.actionBelow && action}
      <div className="scene-body" key={index}>{scene.body}</div>
      {scene.actionBelow && action}
      {index > 0 && <nav className="scene-nav" aria-label="Navegación de escenas">
        <button type="button" className="btn" onClick={() => setIndex(index - 1)}>Escena anterior</button>
      </nav>}
    </section>
  )
}

export function BucketClassScenes({ teaching, diagram }: { teaching: Teaching; diagram: ReactNode }) {
  const [uploaded, setUploaded] = useState(false)
  const [answer, setAnswer] = useState<'bucket' | 'key' | null>(null)

  return <SceneDeck scenes={[
    { label: 'Entiende', title: '¿Dónde guarda S3 tus archivos?', action: { label: 'Probar la subida' }, body: <>
      <div className="class-nubi scene-tip"><Nubi size={44} /><p>{teaching.explanation}</p></div>
      {diagram}
    </> },
    { label: 'Experimenta', title: 'Sube tu primer objeto', actionBelow: true,
      action: uploaded ? { label: 'Ir al repaso' } : { label: 'Subir foto.jpg', run: () => setUploaded(true) },
      body: <>
      <p>Al pulsar «Subir foto.jpg», verás cómo se copia el archivo de tu equipo al bucket. No se sube ningún archivo real.</p>
      <div className="scene-transfer" data-uploaded={uploaded} aria-hidden="true">
        <div className="scene-zone"><span>Tu equipo</span></div>
        <div className="scene-zone scene-destination"><span>Bucket · mi-bucket</span></div>
        <div className="scene-flight"><code>foto.jpg</code></div>
      </div>
      <div className="scene-controls">
        <p role="status">{uploaded ? 'Objeto guardado en mi-bucket.' : ''}</p>
        {uploaded && <button type="button" className="btn" onClick={() => setUploaded(false)}>Reiniciar simulación</button>}
      </div>
      {uploaded && <div className="scene-command">
        <pre className="example-term"><code>aws s3 cp foto.jpg s3://<mark>mi-bucket</mark>/<mark>foto.jpg</mark></code></pre>
        <p><strong>Bucket:</strong> mi-bucket · <strong>Clave del objeto:</strong> foto.jpg</p>
      </div>}
    </> },
    { label: 'Repasa', title: '¿Cuál es la clave del objeto?', body: <>
      <p>Elige una opción. En <code>s3://mi-bucket/foto.jpg</code>, ¿qué parte identifica el objeto dentro del bucket?</p>
      <div className="scene-controls">
        <button type="button" className="btn" aria-pressed={answer === 'bucket'} onClick={() => setAnswer('bucket')}>mi-bucket</button>
        <button type="button" className="btn" aria-pressed={answer === 'key'} onClick={() => setAnswer('key')}>foto.jpg</button>
      </div>
      <p role="status" className="scene-feedback" data-tone={answer === 'key' ? 'success' : answer === 'bucket' ? 'hint' : undefined}>
        {answer === 'key' ? <>
          <span aria-hidden="true">✓</span>
          <span><strong>Correcto:</strong> <code>foto.jpg</code> es la clave del objeto.</span>
        </> : answer === 'bucket' ? <>
          <span aria-hidden="true">↻</span>
          <span><strong>Inténtalo de nuevo.</strong> Ese es el nombre del bucket. Prueba con la parte que viene después de la barra.</span>
        </> : null}
      </p>
      <div className="class-nubi scene-tip"><Nubi size={44} /><p>{teaching.review}</p></div>
    </> },
  ]} />
}

const prefix = 'informes/'
const objectKeys = ['informes/enero.txt', 'informes/febrero.txt', 'borradores/enero.txt']

export function PrefixClassScenes({ teaching, diagram }: { teaching: Teaching; diagram: ReactNode }) {
  const [filtered, setFiltered] = useState(false)
  const [answer, setAnswer] = useState<string | null>(null)
  const visibleKeys = filtered ? objectKeys.filter((key) => key.startsWith(prefix)) : objectKeys
  const correct = answer === 'informes/enero.txt'

  return <SceneDeck scenes={[
    { label: 'Entiende', title: '¿Qué identifica a un objeto en S3?', action: { label: 'Explorar el bucket' }, body: <>
      <div className="class-nubi scene-tip"><Nubi size={44} /><p>{teaching.explanation}</p></div>
      {diagram}
    </> },
    { label: 'Experimenta', title: 'Encuentra los informes', actionBelow: true,
      action: filtered ? { label: 'Ir al repaso' } : { label: 'Filtrar por informes/', run: () => setFiltered(true) },
      body: <>
      <p>Filtra las claves que empiezan por <code>informes/</code>. Es una simulación: no se ejecutan comandos ni se modifican objetos.</p>
      <div className="diagram-bucket">
        <p className="diagram-bucket-name"><span className="s3-chip" aria-hidden="true">S3</span>Bucket: archivo-nubi</p>
        <ul className="diagram-objects" aria-label="Claves del bucket archivo-nubi">
          {visibleKeys.map((key) => <li key={key}><code>{key}</code></li>)}
        </ul>
      </div>
      <div className="scene-controls">
        <p role="status">{filtered ? `${visibleKeys.length} de ${objectKeys.length} claves coinciden. El filtro no borra objetos.` : `Se muestran las ${objectKeys.length} claves.`}</p>
        {filtered && <button type="button" className="btn" onClick={() => setFiltered(false)}>Mostrar todas las claves</button>}
      </div>
      {filtered && <div className="scene-command">
        <pre className="example-term"><code>aws s3 ls s3://archivo-nubi/{prefix}</code></pre>
        <p>La lista solo muestra objetos cuyo nombre completo empieza con el prefijo indicado.</p>
      </div>}
    </> },
    { label: 'Repasa', title: 'Mismo nombre, distinta clave', body: <>
      <p>Hay un <code>enero.txt</code> en informes y otro en borradores. ¿Con qué clave identificarías el de informes?</p>
      <div className="scene-controls">
        {['enero.txt', 'informes/enero.txt'].map((key) => (
          <button key={key} type="button" className="btn" aria-pressed={answer === key} onClick={() => setAnswer(key)}>{key}</button>
        ))}
      </div>
      <p role="status" className="scene-feedback" data-tone={answer === null ? undefined : correct ? 'success' : 'hint'}>
        {answer !== null && <>
          <span aria-hidden="true">{correct ? '✓' : '↻'}</span>
          <span>{correct ? <><strong>Correcto:</strong> <code>informes/enero.txt</code> incluye el prefijo y el nombre.</>
            : <><strong>Inténtalo de nuevo.</strong> El nombre solo no distingue los dos objetos. Incluye el prefijo.</>}</span>
        </>}
      </p>
      <div className="class-nubi scene-tip"><Nubi size={44} /><p>{teaching.review}</p></div>
    </> },
  ]} />
}
