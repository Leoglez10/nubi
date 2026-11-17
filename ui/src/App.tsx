import { useEffect, useState } from 'react'
import { api, type Content } from './api'
import { SHELL_SCREENS } from './nav'
import { Shell } from './shell/Shell'
import { AppProvider, useApp, type AppState } from './store'
import { Screens } from './screens'

/** Loads the course, then hands it to the app. */
export default function App() {
  const [content, setContent] = useState<Content | null>(null)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    api.course().then(setContent, (e) => setError(String(e)))
  }, [])

  if (error) return <p role="alert" className="boot">No se pudo cargar el curso: {error}</p>
  if (!content) return <p className="boot">Cargando…</p>
  const initial: AppState = { content, screen: 'intro', stopId: null, setup: null, me: null, accountOpen: false }
  return (
    <AppProvider initial={initial}>
      <Root />
    </AppProvider>
  )
}

function Root() {
  const { state, set } = useApp()

  // The first setup probe runs in the background; the sidebar shows its result.
  useEffect(() => {
    api.probe().then((setup) => set({ setup }), () => {})
  }, [set])

  return (
    <div className="app">
      {SHELL_SCREENS.includes(state.screen) ? <Shell><Screens /></Shell> : <Screens />}
    </div>
  )
}
