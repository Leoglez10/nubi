// Ajustes (src/view/settings.rs): the signed-in profile (edit or delete it) and preferences
// that apply everywhere.
import { useId, useRef, useState, type FormEvent } from 'react'
import { api, type Profile } from '../api'
import { Avatar } from '../art'
import { useApp } from '../store'
import { ColorChoice, knownColor } from './ColorChoice'
import './Settings.css'

function systemReducedMotion() {
  return typeof matchMedia === 'function' && matchMedia('(prefers-reduced-motion: reduce)').matches
}

export function Settings() {
  const { state, set, go } = useApp()
  const [note, setNote] = useState<string | null>(null)
  const profile = state.me?.profile ?? null

  const save = async (changed: Profile) => {
    try {
      set({ me: await api.updateProfile(changed) })
      setNote(null)
      return true
    } catch (e) {
      setNote(`No se pudieron guardar los cambios: ${e}`)
      return false
    }
  }

  return (
    <section className="page settings">
      <header className="settings-head">
        <div>
          <h1 id="page-title" tabIndex={-1}>Ajustes</h1>
          <p>{profile ? 'Tu perfil y tus preferencias, guardados en esta computadora.' : 'Preferencias para esta sesión.'}</p>
        </div>
        <button type="button" className="btn" onClick={() => go(profile ? 'home' : 'profiles')}>Volver</button>
      </header>
      {profile && <ProfileSection key={profile.id} profile={profile} onSave={save} />}
      <section className="settings-section card" aria-labelledby="settings-accessibility">
        <h2 id="settings-accessibility">Accesibilidad</h2>
        <MotionRow profile={profile} onSave={save} />
      </section>
      {note && <p className="note error" role="alert">{note}</p>}
      {profile && (
        <DangerSection profile={profile} onError={setNote}
          onDeleted={() => go('profiles', { me: null, stopId: null })} />
      )}
    </section>
  )
}

/** Name and color of the signed-in profile, saved with one button. */
function ProfileSection({ profile, onSave }: { profile: Profile; onSave: (p: Profile) => Promise<boolean> }) {
  const [name, setName] = useState(profile.name)
  const [color, setColor] = useState(knownColor(profile.color))
  const [invalid, setInvalid] = useState(false)
  const errorId = useId()
  const input = useRef<HTMLInputElement>(null)
  const trimmed = name.trim()
  const changed = trimmed !== profile.name || color !== profile.color

  const submit = async (e: FormEvent) => {
    e.preventDefault()
    if (!trimmed) {
      setInvalid(true)
      input.current?.focus()
      return
    }
    await onSave({ ...profile, name: trimmed, color })
  }

  return (
    <section className="settings-section card" aria-labelledby="settings-profile">
      <h2 id="settings-profile">Perfil</h2>
      <form className="settings-body profile-edit" noValidate onSubmit={submit}>
        <div className="profile-edit-head">
          <Avatar name={trimmed || '?'} color={color} size={48} />
          <div>
            <strong>{profile.name}</strong>
            <span>Perfil local</span>
          </div>
        </div>
        <div className="field">
          <label className="field">
            <span className="field-label">Nombre</span>
            <input ref={input} type="text" name="name" value={name} maxLength={40} required autoComplete="off"
              aria-invalid={invalid} aria-describedby={invalid ? errorId : undefined}
              onChange={(e) => { setName(e.target.value); setInvalid(false) }} />
          </label>
          {invalid && <span id={errorId} className="field-error">Escribe tu nombre.</span>}
        </div>
        <ColorChoice value={color} onChange={setColor} />
        <div className="form-actions">
          <button type="submit" className="btn btn-primary" disabled={!changed}>Guardar cambios</button>
        </div>
      </form>
    </section>
  )
}

/** Reduced motion, saved to the profile. Locked on when the system already asks for it. */
function MotionRow({ profile, onSave }: { profile: Profile | null; onSave: (p: Profile) => Promise<boolean> }) {
  const system = systemReducedMotion()
  const on = system || (profile?.reduced_motion ?? false)
  const id = useId()
  return (
    <div className="settings-body setting-row">
      <div className="setting-text">
        <label htmlFor={id}><strong>Reducir movimiento</strong></label>
        <p id={`${id}-detail`}>
          {system
            ? 'Tu sistema ya pide movimiento reducido, así que está activo en toda la app.'
            : 'Quita las animaciones al cambiar de pantalla y al pulsar botones.'}
        </p>
      </div>
      {/* `switch` is the native toggle where supported; role="switch" keeps the semantics elsewhere. */}
      <input id={id} type="checkbox" role="switch" className="switch" {...{ switch: '' }}
        aria-describedby={`${id}-detail`} checked={on} disabled={system || !profile}
        onChange={(e) => profile && onSave({ ...profile, reduced_motion: e.target.checked })} />
    </div>
  )
}

/** Deleting asks once more, naming what is lost. */
function DangerSection({ profile, onDeleted, onError }: {
  profile: Profile
  onDeleted: () => void
  onError: (msg: string) => void
}) {
  const dialog = useRef<HTMLDialogElement>(null)
  const titleId = useId()

  const answer = async (e: FormEvent<HTMLFormElement>) => {
    e.preventDefault()
    const choice = ((e.nativeEvent as SubmitEvent).submitter as HTMLButtonElement | null)?.value ?? ''
    dialog.current?.close(choice)
    if (choice !== 'delete') return
    try {
      await api.deleteProfile(profile.id)
      onDeleted()
    } catch (err) {
      onError(`No se pudo eliminar el perfil: ${err}`)
    }
  }

  return (
    <section className="settings-section card" aria-labelledby="settings-danger">
      <h2 id="settings-danger">Zona de peligro</h2>
      <div className="settings-body setting-row">
        <div className="setting-text">
          <strong>Eliminar perfil</strong>
          <p>Borra este perfil y su progreso de esta computadora.</p>
        </div>
        <button type="button" className="btn danger-outline" onClick={() => dialog.current?.showModal()}>Eliminar perfil</button>
      </div>
      <dialog ref={dialog} className="confirm card" aria-labelledby={titleId}>
        <h2 id={titleId}>¿Eliminar a {profile.name}?</h2>
        <p>Se borrarán su perfil y todo su progreso. No se puede deshacer.</p>
        <form method="dialog" className="form-actions" onSubmit={answer}>
          <button type="submit" value="cancel" className="btn" autoFocus>Cancelar</button>
          <button type="submit" value="delete" className="btn btn-danger">Sí, eliminar</button>
        </form>
      </dialog>
    </section>
  )
}
