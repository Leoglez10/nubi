// Where the student is and where they can go: pure functions over content and progress.
import { classes, type Content, type Progress, type Teaching, type Node } from './api'

export type Screen = 'intro' | 'setup' | 'profiles' | 'home' | 'class' | 'lesson' | 'lab' | 'settings'

/** Screens inside the sidebar + top bar frame. */
export const SHELL_SCREENS: Screen[] = ['home', 'class', 'lesson', 'lab', 'settings']

export const completed = (progress: Progress | undefined, id: string) => (progress?.nodes[id]?.best_stars ?? 0) > 0

export type ClassRef = { node: Node; teaching: Teaching }

/** The first class whose lab is not done yet, else the last one. */
export function currentClass(content: Content, progress?: Progress): ClassRef | undefined {
  const all = classes(content)
  return all.find((c) => !completed(progress, c.teaching.lab)) ?? all.at(-1)
}

/** The class a lesson or lab id belongs to. */
export function classOf(content: Content, id: string | null): ClassRef | undefined {
  return id == null ? undefined : classes(content).find((c) => c.node.id === id || c.teaching.lab === id)
}

/** The top bar's location text. */
export function crumb(screen: Screen, shown?: ClassRef): string {
  if (screen === 'settings') return 'Ajustes'
  const c = shown && `Clase ${shown.teaching.number} · ${shown.teaching.title}`
  if (!c) return 'Inicio'
  if (screen === 'class') return c
  if (screen === 'lesson') return `${c} · Práctica`
  if (screen === 'lab') return `${c} · Laboratorio`
  return 'Inicio'
}
