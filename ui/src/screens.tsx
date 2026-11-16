// Routes each screen to its view. Screens still marked "(pendiente)" arrive in later tasks.
import { classOf } from './nav'
import { useApp } from './store'
import { Class } from './screens/Class'
import { Home } from './screens/Home'
import { Intro } from './screens/Intro'
import { Lesson } from './screens/Lesson'
import { Profiles } from './screens/Profiles'
import { Settings } from './screens/Settings'
import { Setup } from './screens/Setup'

export function Screens() {
  const { state } = useApp()
  switch (state.screen) {
    case 'intro': return <Intro />
    case 'setup': return <Setup />
    case 'profiles': return <Profiles />
    case 'settings': return <Settings />
    case 'home': return <Home />
    case 'class': return <Class />
    case 'lesson': return <Lesson />
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
