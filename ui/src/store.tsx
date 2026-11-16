// App state shared by every screen, and navigation with a short content transition.
import { createContext, useCallback, useContext, useMemo, useState, type ReactNode } from 'react'
import { flushSync } from 'react-dom'
import type { Content, LabView, LessonView, ProgressView, SetupView } from './api'
import type { Screen } from './nav'

export interface AppState {
  content: Content
  screen: Screen
  /** The class, lesson or lab id the current screen shows. */
  stopId: string | null
  setup: SetupView | null
  me: ProgressView | null
  accountOpen: boolean
  /** The practice in progress, kept across navigation so the student can come back to it. */
  lesson?: LessonState | null
  /** The lab in progress, kept across navigation like `lesson`. */
  lab?: LabState | null
}

/** What the student has picked on the current exercise, before checking it. */
export interface LessonDraft {
  choice: number | null
  command: string
  /** Order: display positions in the order placed. */
  picked: number[]
  /** Match: [left, right] content indices. */
  pairs: [number, number][]
  /** Match: the concept waiting for its meaning. */
  left: number | null
}

export interface LessonState {
  view: LessonView
  /** Display order of the current order/match exercise (from `api.shuffled`), else empty. */
  display: number[]
  draft: LessonDraft
  /** True while Clase was opened from this practice and should offer the way back. */
  reviewing: boolean
}

export interface LabState {
  view: LabView
  /** What is typed at the prompt, not sent yet. */
  input: string
  /** The line sent and waiting for the engine, shown with "corriendo…". */
  running: string | null
  /** Clase was opened from this lab ("Ver explicación" or "Repasar la clase") and should offer the way back. */
  reviewing: 'explain' | 'review' | null
  /** Coming back from Clase: show this attempt as it was, even when already complete. */
  resume: boolean
}

interface Store {
  state: AppState
  /** Merges a change without animating. */
  set: (patch: Partial<AppState>) => void
  /** Changes screen; the content area slides in unless motion is reduced. */
  go: (screen: Screen, patch?: Partial<AppState>) => void
  reducedMotion: boolean
}

const Ctx = createContext<Store | null>(null)

export function useApp(): Store {
  const store = useContext(Ctx)
  if (!store) throw new Error('useApp outside <AppProvider>')
  return store
}

function systemReducedMotion() {
  return typeof matchMedia === 'function' && matchMedia('(prefers-reduced-motion: reduce)').matches
}

export function AppProvider({ initial, children }: { initial: AppState; children: ReactNode }) {
  const [state, setState] = useState(initial)
  const set = useCallback((patch: Partial<AppState>) => setState((s) => ({ ...s, ...patch })), [])
  const reducedMotion = (state.me?.profile.reduced_motion ?? false) || systemReducedMotion()

  const go = useCallback((screen: Screen, patch: Partial<AppState> = {}) => {
    const update = () => setState((s) => ({ ...s, accountOpen: false, ...patch, screen }))
    // View transitions are progressive: without them, or with reduced motion, swap at once.
    if (!document.startViewTransition || reducedMotion) {
      update()
      return
    }
    document.startViewTransition(() => flushSync(update))
  }, [reducedMotion])

  const store = useMemo(() => ({ state, set, go, reducedMotion }), [state, set, go, reducedMotion])
  return <Ctx.Provider value={store}>{children}</Ctx.Provider>
}
