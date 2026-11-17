import { render, screen, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { clearMocks, mockIPC } from '@tauri-apps/api/mocks'
import { afterEach, expect, test } from 'vitest'
import type { LessonView, ProgressView } from '../api'
import { Screens } from '../screens'
import { Shell } from '../shell/Shell'
import { AppProvider, useApp, type AppState } from '../store'
import { content, me } from '../test/fixtures'

afterEach(clearMocks)

type Session = LessonView['session']

/** The engine's view of the first class's practice at exercise `index`. */
function at(index: number, session: Partial<Session> = {}, rest: Partial<LessonView> = {}): LessonView {
  return {
    id: 's3-buckets',
    session: { index, total: 4, errors: 0, mistakes: [], feedback: null, ...session },
    finished: false,
    progress: index / 4,
    saved: null,
    ...rest,
  }
}

function backend(replies: Record<string, unknown>) {
  const calls: { cmd: string; args: Record<string, unknown> }[] = []
  mockIPC((cmd, args) => {
    calls.push({ cmd, args: args as Record<string, unknown> })
    const r = replies[cmd]
    return typeof r === 'function' ? r(args) : r
  })
  return { calls, of: (cmd: string) => calls.filter((c) => c.cmd === cmd) }
}

/** Shows the stars in `me`, to see the store change. */
function MyStars() {
  const stars = useApp().state.me?.progress.nodes['s3-buckets']?.best_stars ?? 0
  return <span data-testid="my-stars">{stars}</span>
}

function renderLesson(profile: ProgressView | null = me(), shell = false) {
  const initial: AppState = { content, screen: 'lesson', stopId: 's3-buckets', setup: null, me: profile, accountOpen: false }
  return render(
    <AppProvider initial={initial}>
      {shell ? <Shell><Screens /></Shell> : <Screens />}
      <MyStars />
    </AppProvider>,
  )
}

/** The page heading once the practice has loaded (it reads "Práctica" while loading). */
const prompt = () => screen.findByRole('heading', { level: 1, name: (name) => name !== 'Práctica' })

test('choice builds a choice answer with the picked option', async () => {
  const user = userEvent.setup()
  const b = backend({ lesson_start: at(0), lesson_check: at(0, { feedback: { type: 'right' } }) })
  renderLesson()
  expect(await prompt()).toHaveTextContent('¿Qué guarda un bucket?')
  expect(screen.getByText('ELIGE LA RESPUESTA CORRECTA')).toBeInTheDocument()
  expect(screen.getByRole('button', { name: 'Comprobar' })).toBeDisabled()
  expect(screen.getByText('Elige una respuesta para comprobar.')).toBeInTheDocument()

  await user.click(screen.getByRole('radio', { name: /Servidores/ }))
  expect(screen.getByText('¿Lista tu respuesta? Compruébala.')).toBeInTheDocument()
  await user.click(screen.getByRole('button', { name: 'Comprobar' }))
  expect(b.of('lesson_check')[0].args.answer).toEqual({ type: 'choice', value: 1 })
})

test('command builds a trimmed command answer and Enter checks it', async () => {
  const user = userEvent.setup()
  const b = backend({ lesson_start: at(1), lesson_check: at(1, { feedback: { type: 'right' } }) })
  renderLesson()
  expect(await prompt()).toHaveTextContent('Lista tus buckets')
  await user.type(screen.getByRole('textbox', { name: 'Tu comando' }), '  aws s3 ls {Enter}')
  expect(b.of('lesson_check')[0].args.answer).toEqual({ type: 'command', value: 'aws s3 ls' })
})

test('order maps the shuffled display back to content indices', async () => {
  const user = userEvent.setup()
  const b = backend({ lesson_start: at(2), shuffled: [1, 2, 0], lesson_check: at(2, { feedback: { type: 'right' } }) })
  renderLesson()
  expect(await prompt()).toHaveTextContent('Ordena los pasos para subir un archivo')
  expect(b.of('shuffled')[0].args).toEqual({ n: 3 })
  const pool = () => within(screen.getByRole('list', { name: 'Pasos por colocar' }))
  // Shown as steps 1, 2, 0.
  expect(pool().getAllByRole('button').map((x) => x.textContent)).toEqual(['Subir el archivo', 'Listar el bucket', 'Crear el bucket'])

  await user.click(pool().getByRole('button', { name: 'Subir el archivo' }))
  // A placed step goes back to the pool when tapped.
  await user.click(screen.getByRole('button', { name: '1. Subir el archivo' }))
  for (const step of ['Crear el bucket', 'Subir el archivo', 'Listar el bucket']) {
    await user.click(pool().getByRole('button', { name: step }))
  }
  await user.click(screen.getByRole('button', { name: 'Comprobar' }))
  expect(b.of('lesson_check')[0].args.answer).toEqual({ type: 'order', value: [0, 1, 2] })
})

test('match pairs content indices whatever the display order', async () => {
  const user = userEvent.setup()
  const b = backend({ lesson_start: at(3), shuffled: [1, 2, 0], lesson_check: at(3, { feedback: { type: 'right' } }) })
  renderLesson()
  expect(await prompt()).toHaveTextContent('Une cada concepto con su significado')
  const left = within(screen.getByRole('list', { name: 'Conceptos' }))
  const right = within(screen.getByRole('list', { name: 'Significados' }))
  expect(right.getAllByRole('button').map((x) => x.textContent)).toEqual(['Archivo', 'Nombre', 'Contenedor'])

  for (const [concept, meaning] of [['Bucket', 'Nombre'], ['Bucket', 'Contenedor'], ['Objeto', 'Archivo'], ['Clave', 'Nombre']]) {
    await user.click(left.getByRole('button', { name: new RegExp(`^${concept}`) }))
    await user.click(right.getByRole('button', { name: new RegExp(`^${meaning}`) }))
  }
  await user.click(screen.getByRole('button', { name: 'Comprobar' }))
  expect(b.of('lesson_check')[0].args.answer).toEqual({ type: 'match', value: [[0, 0], [1, 1], [2, 2]] })
})

test('a wrong answer shows the hint and "Intentar de nuevo" retries', async () => {
  const user = userEvent.setup()
  const b = backend({
    lesson_start: at(0),
    lesson_check: at(0, { errors: 1, feedback: { type: 'wrong', hint: 'Piensa en archivos.' } }),
    lesson_retry: at(0, { errors: 1 }),
  })
  renderLesson()
  await user.click(await screen.findByRole('radio', { name: /Usuarios/ }))
  await user.click(screen.getByRole('button', { name: 'Comprobar' }))
  const status = await screen.findByRole('status')
  expect(status).toHaveTextContent('Casi. Inténtalo otra vez, sin límite.')
  expect(status).toHaveTextContent('Pista: Piensa en archivos.')
  expect(screen.getByRole('radio', { name: /Servidores/ })).toBeDisabled()

  await user.click(screen.getByRole('button', { name: 'Intentar de nuevo' }))
  expect(b.of('lesson_retry')).toHaveLength(1)
  expect(await screen.findByRole('button', { name: 'Comprobar' })).toBeDisabled()
  expect(screen.getByRole('radio', { name: /Usuarios/ })).not.toBeChecked()
})

test('a right answer then "Continuar" moves to the next exercise', async () => {
  const user = userEvent.setup()
  const b = backend({ lesson_start: at(0), lesson_check: at(0, { feedback: { type: 'right' } }), lesson_next: at(1) })
  renderLesson()
  expect(await screen.findByText('Pregunta 1 de 4')).toBeInTheDocument()
  await user.click(screen.getByRole('radio', { name: /Objetos/ }))
  await user.click(screen.getByRole('button', { name: 'Comprobar' }))
  expect(await screen.findByRole('status')).toHaveTextContent('¡Correcto!')
  await user.click(screen.getByRole('button', { name: 'Continuar' }))
  expect(b.of('lesson_next')).toHaveLength(1)
  expect(await screen.findByText('Pregunta 2 de 4')).toBeInTheDocument()
  expect(await prompt()).toHaveTextContent('Lista tus buckets')
})

test('finishing shows stars, errors and mistakes and updates `me` from `saved`', async () => {
  const user = userEvent.setup()
  const mistakes = ['¿Qué guarda un bucket?', 'Lista tus buckets', 'Une cada concepto con su significado']
  const done = at(4, { errors: 3, mistakes }, { finished: true, progress: 1, saved: me({ 's3-buckets': 1 }) })
  backend({ lesson_start: at(3), shuffled: [0, 1, 2], lesson_check: at(3, { errors: 3, mistakes, feedback: { type: 'right' } }), lesson_next: done })
  renderLesson()
  await prompt()
  for (const name of ['Bucket', 'Contenedor', 'Objeto', 'Archivo', 'Clave', 'Nombre']) {
    await user.click(screen.getByRole('button', { name: new RegExp(`^${name}`) }))
  }
  await user.click(screen.getByRole('button', { name: 'Comprobar' }))
  await user.click(await screen.findByRole('button', { name: 'Continuar' }))

  expect(await prompt()).toHaveTextContent('¡Lección completada!')
  expect(screen.getByRole('img', { name: '1 de 3 estrellas' })).toHaveTextContent('★☆☆')
  expect(screen.getByText('Tuviste 3 errores. 3 estrellas = cero errores.')).toBeInTheDocument()
  const review = within(screen.getByRole('region', { name: 'Para repasar' }))
  expect(review.getAllByRole('listitem').map((li) => li.textContent)).toEqual(mistakes)
  expect(screen.getByTestId('my-stars')).toHaveTextContent('1')
  expect(screen.getByText('Tu resultado quedó guardado.')).toBeInTheDocument()
  expect(screen.getByRole('button', { name: 'Perfeccionar' })).toBeInTheDocument()
  await user.click(screen.getByRole('button', { name: 'Volver al inicio' }))
  expect(await prompt()).toHaveTextContent('Hola, Alex')
})

test('reopening Práctica from the sidebar on the same unfinished stop keeps the session', async () => {
  const user = userEvent.setup()
  const b = backend({ lesson_start: at(1), progress: null })
  renderLesson(me(), true)
  await user.type(await screen.findByRole('textbox', { name: 'Tu comando' }), 'aws s3')
  const nav = within(screen.getByRole('navigation', { name: 'Principal' }))
  await user.click(nav.getByRole('button', { name: 'Inicio' }))
  expect(await prompt()).toHaveTextContent('Hola, Alex')
  await user.click(nav.getByRole('button', { name: 'Práctica' }))

  expect(await prompt()).toHaveTextContent('Lista tus buckets')
  expect(screen.getByRole('textbox', { name: 'Tu comando' })).toHaveValue('aws s3')
  expect(b.of('lesson_start')).toHaveLength(1)
})

test('going to the class from the practice and back keeps the exercise', async () => {
  const user = userEvent.setup()
  const b = backend({ lesson_start: at(0) })
  renderLesson()
  await user.click(await screen.findByRole('radio', { name: /Servidores/ }))
  await user.click(screen.getByRole('button', { name: 'Ver explicación' }))

  expect(await prompt()).toHaveTextContent('Buckets y objetos')
  expect(screen.queryByRole('button', { name: 'Empezar práctica' })).toBeNull()
  await user.click(screen.getByRole('button', { name: 'Volver a la práctica' }))

  expect(await prompt()).toHaveTextContent('¿Qué guarda un bucket?')
  expect(screen.getByRole('radio', { name: /Servidores/ })).toBeChecked()
  expect(b.of('lesson_start')).toHaveLength(1)
})
