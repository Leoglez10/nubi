// Dev only: in a plain browser (no Tauri) answer commands with sample data, so layouts can be
// checked at any width with `npm run dev`. Never bundled into the app (see main.tsx).
import { mockIPC } from '@tauri-apps/api/mocks'
import type { Answer, Exercise, LabView, LessonView } from './api'
import { content, me } from './test/fixtures'

export function previewWithSampleData() {
  const profile = me({ 's3-buckets': 3 })
  const replies: Record<string, unknown> = {
    course: content,
    profiles: [{ profile: profile.profile, stars: 3 }],
    select_profile: profile,
    probe: {
      report: { docker: true, docker_running: true, aws_cli: true, emulator: true },
      needs: [],
      ready: true,
      os: 'mac',
    },
    fix: 'Vista previa: aquí se iniciaría el emulador.',
    create_profile: profile,
    update_profile: profile,
  }
  const lesson = sampleLesson(profile)
  const lab = sampleLab(profile)
  mockIPC((cmd, args) => {
    const a = args as Record<string, unknown>
    return lesson(cmd, a) ?? lab(cmd, a) ?? replies[cmd] ?? null
  })
}

/** A rough stand-in for the engine's lesson session, enough to click through a practice. */
function sampleLesson(profile: ReturnType<typeof me>) {
  let id = ''
  let index = 0
  let errors = 0
  let mistakes: string[] = []
  let feedback: LessonView['session']['feedback'] = null
  const exercises = () => content.nodes[id]?.exercises ?? []
  const view = (): LessonView => {
    const total = exercises().length
    const finished = index >= total
    return {
      id,
      session: { index, total, errors, mistakes, feedback },
      finished,
      progress: total ? index / total : 1,
      saved: finished ? profile : null,
    }
  }
  const right = (ex: Exercise, a: Answer) => {
    if (ex.type === 'choice' && a.type === 'choice') return a.value === ex.answer
    if (ex.type === 'command' && a.type === 'command') return ex.answers.includes(a.value)
    if (ex.type === 'order' && a.type === 'order') return a.value.every((v, i) => v === i)
    if (ex.type === 'match' && a.type === 'match') return a.value.every(([l, r]) => l === r)
    return false
  }
  return (cmd: string, args: Record<string, unknown>): unknown => {
    switch (cmd) {
      case 'shuffled': {
        const n = args.n as number
        return Array.from({ length: n }, (_, i) => (i + 1) % n)
      }
      case 'lesson_start':
        id = args.id as string
        index = 0; errors = 0; mistakes = []; feedback = null
        return view()
      case 'lesson_check': {
        const ex = exercises()[index]
        if (right(ex, args.answer as Answer)) feedback = { type: 'right' }
        else {
          errors += 1
          if (!mistakes.includes(ex.prompt)) mistakes = [...mistakes, ex.prompt]
          feedback = { type: 'wrong', hint: ex.hint }
        }
        return view()
      }
      case 'lesson_retry':
        feedback = null
        return view()
      case 'lesson_next':
        index += 1
        feedback = null
        return view()
      default:
        return undefined
    }
  }
}

/** A tiny stand-in for the lab terminal: built-ins, and `aws` lines that tick the next objective. */
function sampleLab(profile: ReturnType<typeof me>) {
  type Lines = LabView['session']['lines']
  let id = ''
  let lines: Lines = []
  let done: boolean[] = []
  let errors = 0
  let mistakes: LabView['session']['mistakes'] = []
  let history: string[] = []
  let cursor: number | null = null
  const objectives = () => content.nodes[id]?.objectives ?? []
  const view = (saved = false): LabView => ({
    id,
    session: { lines, done, errors, mistakes },
    complete: done.length > 0 && done.every(Boolean),
    saved: saved ? profile : null,
  })
  const say = (kind: Lines[number]['kind'], text: string) => { lines = [...lines, { kind, text }] }
  return (cmd: string, args: Record<string, unknown>): unknown => {
    switch (cmd) {
      case 'lab_start':
        id = args.id as string
        lines = []
        say('info', 'Nubi · terminal conectada a tu AWS Emulator local')
        say('info', 'Escribe "ayuda" para ver qué puedes hacer, o "pista" si te atoras.')
        done = objectives().map(() => false)
        errors = 0; mistakes = []; history = []; cursor = null
        return view()
      case 'lab_submit': {
        const line = (args.line as string).trim().split(/\s+/).join(' ')
        if (!line) return view()
        cursor = null
        if (line === 'limpiar') { lines = []; return view() }
        history = [...history, line]
        say('cmd', line)
        const next = done.indexOf(false)
        if (line === 'ayuda') {
          say('info', 'Escribe comandos aws reales: se ejecutan contra tu emulador local.')
          say('info', 'Extras: pista · limpiar · ↑/↓ para repetir comandos.')
        } else if (line === 'pista') {
          say('info', next < 0 ? '¡Ya completaste todos los objetivos!' : `Pista: ${objectives()[next].hint}`)
        } else if (!line.startsWith('aws ')) {
          errors += 1
          mistakes = [...mistakes, { command: line, error: 'Solo se permiten comandos aws.' }]
          say('err', 'Solo se permiten comandos aws.')
        } else if (next >= 0 && line === objectives()[next].hint) {
          done = done.map((d, i) => d || i === next)
          say('out', 'Vista previa: comando ejecutado.')
          say('ok', '¡Objetivo cumplido!')
          return new Promise((r) => setTimeout(() => r(view(done.every(Boolean))), 400))
        } else {
          say('out', 'Vista previa: comando ejecutado.')
        }
        return new Promise((r) => setTimeout(() => r(view()), 400))
      }
      case 'lab_history': {
        if (args.older) {
          if (!history.length) return null
          cursor = cursor == null ? history.length - 1 : Math.max(0, cursor - 1)
          return history[cursor]
        }
        if (cursor == null || cursor + 1 >= history.length) { cursor = null; return null }
        cursor += 1
        return history[cursor]
      }
      default:
        return undefined
    }
  }
}
