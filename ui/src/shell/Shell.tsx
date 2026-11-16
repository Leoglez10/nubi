// The frame around the learning screens: sidebar navigation and a top bar that says where
// you are. Below 900px the sidebar becomes an icon rail.
import { useEffect, useRef, type ReactNode } from 'react'
import { api } from '../api'
import { Avatar, Icon, Nubi, type IconName } from '../art'
import { classOf, completed, crumb, currentClass, type Screen } from '../nav'
import { useApp } from '../store'
import './shell.css'

export function Shell({ children }: { children: ReactNode }) {
  const { state } = useApp()
  const main = useRef<HTMLElement>(null)

  // View transitions do not move focus: hand it to the new page's heading.
  useEffect(() => {
    const title = main.current?.querySelector<HTMLElement>('#page-title')
    title?.focus({ preventScroll: true })
  }, [state.screen, state.stopId])

  return (
    <div className="shell">
      <Sidebar />
      <div className="shell-main">
        <TopBar />
        <main ref={main} className="shell-content" data-testid="shell-content">{children}</main>
      </div>
    </div>
  )
}

function Sidebar() {
  const { state, go, set } = useApp()
  const current = currentClass(state.content, state.me?.progress)
  const labReady = current != null && completed(state.me?.progress, current.node.id)
  const items: { key: Screen; label: string; icon: IconName; enabled: boolean; open: () => void }[] = [
    { key: 'home', label: 'Inicio', icon: 'home', enabled: true, open: () => go('home', { stopId: null }) },
    { key: 'class', label: 'Clase actual', icon: 'book', enabled: current != null, open: () => go('class', { stopId: current!.node.id }) },
    { key: 'lesson', label: 'Práctica', icon: 'check', enabled: current != null, open: () => go('lesson', { stopId: current!.node.id }) },
    { key: 'lab', label: 'Laboratorio', icon: 'term', enabled: labReady || state.screen === 'lab', open: () => go('lab', { stopId: current!.teaching.lab }) },
  ]
  const signOut = async () => {
    await api.signOut()
    go('profiles', { me: null, stopId: null })
  }

  return (
    <nav className="sidebar" aria-label="Principal">
      <div className="sidebar-scroll">
        <div className="brand">
          <Nubi size={34} />
          <div className="brand-text"><strong>Nubi</strong><span>Lab Cloud</span></div>
        </div>
        <p className="nav-heading">Aprender</p>
        <ul className="nav-list">
          {items.map((it) => (
            <li key={it.key}>
              <button type="button" className="nav-item" aria-current={state.screen === it.key ? 'page' : undefined}
                disabled={!it.enabled} onClick={it.open} title={it.label}>
                <Icon name={it.icon} /><span className="nav-label">{it.label}</span>
              </button>
            </li>
          ))}
        </ul>
        <button type="button" className="nav-heading account-toggle" aria-expanded={state.accountOpen}
          onClick={() => set({ accountOpen: !state.accountOpen })}>
          <span className="nav-label">Cuenta</span><Icon name="chevron" size={14} />
        </button>
        {state.accountOpen && (
          <ul className="nav-list" aria-label="Cuenta">
            <li>
              <button type="button" className="nav-item" aria-current={state.screen === 'settings' ? 'page' : undefined}
                onClick={() => go('settings')} title="Ajustes">
                <Icon name="gear" /><span className="nav-label">Ajustes</span>
              </button>
            </li>
            {state.me && (
              <>
                <li>
                  <button type="button" className="nav-item" onClick={() => go('profiles')} title="Cambiar de perfil">
                    <Icon name="users" /><span className="nav-label">Cambiar de perfil</span>
                  </button>
                </li>
                <li>
                  <button type="button" className="nav-item danger" onClick={signOut} title="Cerrar sesión">
                    <Icon name="logout" /><span className="nav-label">Cerrar sesión</span>
                  </button>
                </li>
              </>
            )}
          </ul>
        )}
      </div>
      <div className="sidebar-footer">
        <Environment />
        {state.me && (
          <button type="button" className="account" aria-expanded={state.accountOpen}
            onClick={() => set({ accountOpen: !state.accountOpen })}>
            <Avatar name={state.me.profile.name} color={state.me.profile.color} />
            <span className="account-text nav-label">
              <strong>{state.me.profile.name}</strong>
              <span>Opciones de cuenta</span>
            </span>
          </button>
        )}
      </div>
    </nav>
  )
}

/** What the machine has ready, from the last setup probe. */
function Environment() {
  const r = useApp().state.setup?.report
  const rows: [string, boolean][] = [
    ['Docker', !!r && r.docker && r.docker_running],
    ['AWS CLI', !!r?.aws_cli],
    ['Emulador S3', !!r?.emulator],
  ]
  return (
    <section className="environment" aria-label="Entorno local">
      <p className="environment-title nav-label">Entorno local</p>
      <ul>
        {rows.map(([name, ok]) => (
          <li key={name} title={`${name}: ${ok ? 'listo' : 'falta'}`}>
            <span className={`dot ${ok ? 'ok' : 'missing'}`} aria-hidden="true" />
            <span className="nav-label env-name">{name}</span>
            <span className={`nav-label env-state ${ok ? 'ok' : 'missing'}`}>{ok ? 'Listo' : 'Falta'}</span>
            <span className="visually-hidden">{`${name}: ${ok ? 'listo' : 'falta'}`}</span>
          </li>
        ))}
      </ul>
    </section>
  )
}

function TopBar() {
  const { state } = useApp()
  const shown = ['class', 'lesson', 'lab'].includes(state.screen) ? classOf(state.content, state.stopId) : undefined
  const course = state.content.course.units[0]?.title ?? 'Curso'
  const total = Object.values(state.me?.progress.nodes ?? {}).reduce((n, r) => n + r.best_stars, 0)
  const level = state.me?.level
  return (
    <header className="top-bar">
      <p className="crumbs">
        <span className="crumb-course">{course}</span>
        <span className="crumb-sep" aria-hidden="true">/</span>
        <span className="crumb" data-testid="crumb">{crumb(state.screen, shown)}</span>
      </p>
      {level && (
        <p className="stars">
          <span aria-hidden="true">★ {total} · Nivel {level.number}<span className="level-name"> · {level.name}</span></span>
          <span className="visually-hidden">{`${total} estrellas, nivel ${level.number}: ${level.name}`}</span>
        </p>
      )}
    </header>
  )
}
