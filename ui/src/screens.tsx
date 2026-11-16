// Routes each screen to its view. Screens still marked "(pendiente)" arrive in later tasks.
import { useEffect, useState } from 'react'
import { api, type ProfileCard } from './api'
import { classOf } from './nav'
import { useApp } from './store'

export function Screens() {
  const { state } = useApp()
  switch (state.screen) {
    case 'profiles': return <ProfilesPlaceholder />
    default: return <Pending />
  }
}

function Pending() {
  const { state } = useApp()
  const shown = classOf(state.content, state.stopId)
  return (
    <section className="page">
      <h1 id="page-title" tabIndex={-1}>{shown?.teaching.title ?? 'Inicio'}</h1>
      <p>Pantalla «{state.screen}» (pendiente).</p>
    </section>
  )
}

function ProfilesPlaceholder() {
  const { go } = useApp()
  const [cards, setCards] = useState<ProfileCard[]>([])
  useEffect(() => { api.profiles().then(setCards, () => {}) }, [])
  return (
    <section className="page">
      <h1 id="page-title" tabIndex={-1}>¿Quién aprende hoy?</h1>
      <ul>
        {cards.map((c) => (
          <li key={c.profile.id}>
            <button type="button" onClick={async () => go('home', { me: await api.selectProfile(c.profile.id) })}>
              {c.profile.name} · ★ {c.stars}
            </button>
          </li>
        ))}
      </ul>
    </section>
  )
}
