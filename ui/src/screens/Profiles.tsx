// "¿Quién va a aprender hoy?": local profiles, no passwords.
import { useEffect, useId, useRef, useState, type FormEvent } from 'react'
import { api, type ProfileCard } from '../api'
import { Avatar, Nubi } from '../art'
import { useApp } from '../store'
import { AVATAR_COLORS, ColorChoice } from './ColorChoice'
import './Profiles.css'

/** Stars needed for each level and its name (LEVELS in src/grading.rs). */
const LEVELS: [number, string][] = [
  [0, 'Despegando'],
  [5, 'Explorador'],
  [15, 'Constructor de nubes'],
  [30, 'Arquitecto'],
  [50, 'Arquitecto senior'],
  [75, 'Maestro de la nube'],
]

function levelOf(stars: number): { number: number; name: string } {
  const i = LEVELS.findLastIndex(([min]) => stars >= min)
  return { number: Math.max(i, 0) + 1, name: LEVELS[Math.max(i, 0)][1] }
}

export function Profiles() {
  const { go } = useApp()
  const [cards, setCards] = useState<ProfileCard[] | null>(null)
  const [creating, setCreating] = useState(false)
  const [note, setNote] = useState<string | null>(null)
  const formId = useId()

  useEffect(() => {
    api.profiles().then((c) => {
      setCards(c)
      setCreating(c.length === 0)
    }, (e) => {
      setCards([])
      setNote(String(e))
    })
  }, [])

  const select = async (id: string) => {
    try {
      go('home', { me: await api.selectProfile(id), stopId: null })
    } catch (e) {
      setNote(String(e))
    }
  }

  return (
    <section className="profiles screen-full">
      <div className="profiles-page">
        <header className="profiles-header">
          <div className="brand">
            <Nubi size={34} />
            <div className="brand-text"><strong>Nubi</strong><span>Lab Cloud</span></div>
          </div>
          <button type="button" className="btn" onClick={() => go('settings')}>Ajustes</button>
        </header>
        <div className="profiles-welcome">
          <Nubi size={72} />
          <div>
            <h1 id="page-title" tabIndex={-1}>¿Quién va a aprender hoy?</h1>
            <p>Elige tu perfil o crea uno nuevo.</p>
          </div>
        </div>
        {cards == null ? (
          <p className="profiles-loading" role="status">Cargando perfiles…</p>
        ) : (
          <ul className="profile-cards">
            {cards.map(({ profile, stars }) => {
              const level = levelOf(stars)
              return (
                <li key={profile.id}>
                  <button type="button" className="profile-card card" onClick={() => select(profile.id)}>
                    <Avatar name={profile.name} color={profile.color} size={64} />
                    <strong className="profile-name">{profile.name}</strong>
                    <span className="profile-level">Nivel {level.number} · {level.name}</span>
                    <span className="profile-stars">★ {stars} estrellas</span>
                  </button>
                </li>
              )
            })}
            <li>
              <button type="button" className="profile-card new-profile" aria-expanded={creating} aria-controls={formId}
                onClick={() => setCreating(!creating)}>
                <span className="plus" aria-hidden="true">+</span>
                <strong>Nuevo perfil</strong>
              </button>
            </li>
          </ul>
        )}
        {creating && (
          <NewProfile id={formId} onError={setNote}
            onCreated={(me) => go('home', { me, stopId: null })} />
        )}
        {note && <p className="note error" role="alert">{note}</p>}
        <p className="local-note">
          <span className="dot ok" aria-hidden="true" />
          Todo se guarda en esta computadora: sin cuentas ni contraseñas.
        </p>
      </div>
    </section>
  )
}

/** The name and color form; an empty name is refused like the Rust store refuses it. */
function NewProfile({ id, onCreated, onError }: {
  id: string
  onCreated: (me: Awaited<ReturnType<typeof api.createProfile>>) => void
  onError: (msg: string) => void
}) {
  const [name, setName] = useState('')
  const [color, setColor] = useState(AVATAR_COLORS[0][0])
  const [invalid, setInvalid] = useState(false)
  const [sending, setSending] = useState(false)
  const input = useRef<HTMLInputElement>(null)
  const errorId = useId()
  const trimmed = name.trim()

  const submit = async (e: FormEvent) => {
    e.preventDefault()
    if (!trimmed) {
      setInvalid(true)
      input.current?.focus()
      return
    }
    setSending(true)
    try {
      onCreated(await api.createProfile(trimmed, color))
    } catch (err) {
      onError(`No se pudo crear el perfil: ${err}`)
      setSending(false)
    }
  }

  return (
    <form id={id} className="profile-form card" noValidate onSubmit={submit} aria-labelledby={`${id}-title`}>
      <div className="profile-form-head">
        <Avatar name={trimmed || '?'} color={color} size={48} />
        <div>
          <h2 id={`${id}-title`}>Nuevo perfil</h2>
          <p>Tu nombre y un color para reconocerte.</p>
        </div>
      </div>
      <div className="field">
        <label className="field">
          <span className="field-label">Nombre</span>
          <input ref={input} type="text" name="name" value={name} maxLength={40} required autoComplete="off"
            placeholder="Por ejemplo, Alex" aria-invalid={invalid} aria-describedby={invalid ? errorId : undefined}
            onChange={(e) => { setName(e.target.value); setInvalid(false) }} />
        </label>
        {invalid && <span id={errorId} className="field-error">Escribe tu nombre.</span>}
      </div>
      <ColorChoice value={color} onChange={setColor} />
      <div className="form-actions">
        <button type="submit" className="btn btn-primary" disabled={sending}>Crear y empezar</button>
      </div>
    </form>
  )
}
