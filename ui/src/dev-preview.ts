// Dev only: in a plain browser (no Tauri) answer commands with sample data, so layouts can be
// checked at any width with `npm run dev`. Never bundled into the app (see main.tsx).
import { mockIPC } from '@tauri-apps/api/mocks'
import type { Answer, Exercise, LessonView } from './api'
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
  mockIPC((cmd, args) => lesson(cmd, args as Record<string, unknown>) ?? replies[cmd] ?? null)
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
