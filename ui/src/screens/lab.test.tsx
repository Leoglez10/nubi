import { render, screen, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { clearMocks, mockIPC } from '@tauri-apps/api/mocks'
import { afterEach, expect, test } from 'vitest'
import type { LabView, ProgressView } from '../api'
import { Screens } from '../screens'
import { Shell } from '../shell/Shell'
import { AppProvider, useApp, type AppState } from '../store'
import { content, me } from '../test/fixtures'

afterEach(clearMocks)

type Session = LabView['session']
const WELCOME: Session['lines'] = [
  { kind: 'info', text: 'Nubi · terminal conectada a tu AWS Emulator local' },
  { kind: 'info', text: 'Escribe "ayuda" para ver qué puedes hacer, o "pista" si te atoras.' },
]

/** The engine's view of the first lab. */
function lab(session: Partial<Session> = {}, rest: Partial<LabView> = {}): LabView {
  return {
    id: 'lab-primer-bucket',
    session: { lines: WELCOME, done: [false, false, false], errors: 0, mistakes: [], ...session },
    complete: false,
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

/** Shows the lab's stars in `me`, to see the store change, and the current screen. */
function Probe() {
  const { state } = useApp()
  return <>
    <span data-testid="my-stars">{state.me?.progress.nodes['lab-primer-bucket']?.best_stars ?? 0}</span>
    <span data-testid="screen">{state.screen}</span>
  </>
}

function renderLab(profile: ProgressView | null = me({ 's3-buckets': 2 }), shell = false) {
  const initial: AppState = { content, screen: 'lab', stopId: 'lab-primer-bucket', setup: null, me: profile, accountOpen: false }
  return render(
    <AppProvider initial={initial}>
      {shell ? <Shell><Screens /></Shell> : <Screens />}
      <Probe />
    </AppProvider>,
  )
}

const log = () => within(screen.getByRole('log', { name: 'Salida de la terminal' }))
const goals = () => within(screen.getByRole('region', { name: 'Objetivos' }))
const prompt = () => screen.findByRole('textbox', { name: 'Comando' })

test('opening the lab starts it and shows the objectives and the first lines', async () => {
  const b = backend({ lab_start: lab() })
  renderLab()
  await prompt()
  expect(b.of('lab_start')[0].args).toEqual({ id: 'lab-primer-bucket' })
  expect(screen.getByText('LABORATORIO')).toBeInTheDocument()
  expect(screen.getByRole('heading', { level: 1 })).toHaveTextContent('Lab 1')
  expect(screen.getByText('Sin errores')).toBeInTheDocument()
  expect(goals().getAllByRole('listitem').map((li) => li.querySelector('.goal-text')!.textContent))
    .toEqual(['Crea el bucket mi-primer-bucket', 'Activa el versionado', 'Sube el archivo hola.txt'])
  expect(log().getByText('Nubi · terminal conectada a tu AWS Emulator local')).toBeInTheDocument()
  expect(screen.getByText('aws → 127.0.0.1:9297')).toBeInTheDocument()
})

test('the DynamoDB lab suggests only applicable commands and expands its table name', async () => {
  const view = lab({}, { table: 'nubi-mensajes-prueba' })
  backend({ lab_start: view })
  const node = content.nodes[view.id]
  const course = { ...content, nodes: { ...content.nodes, [view.id]: { ...node, objectives: [
    { text: 'Crea tu tabla', hint: 'aws dynamodb create-table --table-name {table}', check: { type: 'dynamo_table' as const } },
  ] } } }
  render(<AppProvider initial={{ content: course, screen: 'lab', stopId: view.id, setup: null, me: me(), accountOpen: false }}><Screens /></AppProvider>)
  const input = await prompt()
  expect(screen.queryByRole('button', { name: 'aws s3 ls' })).not.toBeInTheDocument()
  await userEvent.click(screen.getByRole('button', { name: 'siguiente paso' }))
  expect(input).toHaveValue('aws dynamodb create-table --table-name nubi-mensajes-prueba')
})

test('Enter sends the typed line and the lines come back colored by kind', async () => {
  const user = userEvent.setup()
  const lines: Session['lines'] = [
    ...WELCOME,
    { kind: 'cmd', text: 'aws s3 ls' },
    { kind: 'out', text: '2026-11-16 mi-primer-bucket' },
    { kind: 'err', text: 'An error occurred (NoSuchBucket)' },
    { kind: 'ok', text: '¡Objetivo cumplido!' },
  ]
  const b = backend({ lab_start: lab(), lab_submit: lab({ lines }) })
  renderLab()
  const input = await prompt()
  await user.type(input, 'aws s3 ls{Enter}')

  expect(b.of('lab_submit')[0].args).toEqual({ line: 'aws s3 ls' })
  expect(await log().findByText('2026-11-16 mi-primer-bucket')).toHaveClass('term-line', 'out')
  expect(log().getByText('aws s3 ls').closest('li')).toHaveClass('cmd')
  expect(log().getByText('aws s3 ls').closest('li')).toHaveTextContent('nubi@labcloud $ aws s3 ls')
  expect(log().getByText('An error occurred (NoSuchBucket)')).toHaveClass('err')
  expect(log().getByText('¡Objetivo cumplido!')).toHaveClass('ok')
  expect(log().getByText('Nubi · terminal conectada a tu AWS Emulator local')).toHaveClass('info')
  expect(input).toHaveValue('')
  expect(input).toHaveFocus()
})

test('objective ticks follow `done`', async () => {
  const user = userEvent.setup()
  backend({ lab_start: lab(), lab_submit: lab({ done: [true, false, false] }) })
  renderLab()
  await user.type(await prompt(), 'aws s3api create-bucket --bucket mi-primer-bucket{Enter}')
  const [first, second] = goals().getAllByRole('listitem')
  await within(first).findByText('(hecho)')
  expect(first).toHaveClass('done')
  expect(first).toHaveTextContent('✓')
  expect(second).not.toHaveClass('done')
  expect(second).toHaveTextContent('2')
})

test('↑ asks for an older command and puts it at the prompt', async () => {
  const user = userEvent.setup()
  const b = backend({ lab_start: lab(), lab_history: 'aws s3 ls' })
  renderLab()
  const input = await prompt()
  await user.click(input)
  await user.keyboard('{ArrowUp}')
  expect(b.of('lab_history')[0].args).toEqual({ older: true })
  expect(await prompt()).toHaveValue('aws s3 ls')
})

test('a lab that cannot start says why and offers the way to setup', async () => {
  const user = userEvent.setup()
  backend({ lab_start: () => { throw 'El emulador no está corriendo.' } })
  renderLab()
  expect(await screen.findByRole('alert')).toHaveTextContent('El emulador no está corriendo.')
  expect(screen.getByRole('heading', { level: 1 })).toHaveTextContent('Lab 1')
  await user.click(screen.getByRole('button', { name: 'Revisar el entorno' }))
  expect(screen.getByTestId('screen')).toHaveTextContent('setup')
})

test('completing shows the result and updates `me` from `saved`', async () => {
  const user = userEvent.setup()
  const mistakes = [{ command: 'aws s3 cp hola.txt s3://otro/', error: 'NoSuchBucket' }]
  const done = lab({ done: [true, true, true], errors: 1, mistakes }, { complete: true, saved: me({ 's3-buckets': 2, 'lab-primer-bucket': 2 }) })
  backend({ lab_start: lab({ done: [true, true, false], errors: 1 }), lab_submit: done })
  renderLab()
  await user.type(await prompt(), 'aws s3 cp hola.txt s3://mi-primer-bucket/{Enter}')

  const result = within(await screen.findByRole('region', { name: '¡Misión completada!' }))
  expect(result.getByRole('img', { name: '2 de 3 estrellas' })).toHaveTextContent('★★☆')
  expect(result.getByText('Tuviste 1 error. Repítelo desde cero para lograr la perfección.')).toBeInTheDocument()
  expect(result.getByText('aws s3 cp hola.txt s3://otro/')).toBeInTheDocument()
  expect(result.getByText('NoSuchBucket')).toBeInTheDocument()
  expect(result.getByText('Tu mejor marca: ★★☆ · 0 errores')).toBeInTheDocument()
  expect(result.getByText('Tu resultado quedó guardado.')).toBeInTheDocument()
  expect(result.getByRole('button', { name: 'Perfeccionar' })).toBeInTheDocument()
  expect(screen.getByTestId('my-stars')).toHaveTextContent('2')
  await user.click(result.getByRole('button', { name: 'Volver al inicio' }))
  expect(screen.getByTestId('screen')).toHaveTextContent('home')
})

test('reopening Laboratorio from the sidebar on the same unfinished lab keeps the attempt', async () => {
  const user = userEvent.setup()
  const b = backend({ lab_start: lab(), progress: null })
  renderLab(me({ 's3-buckets': 2 }), true)
  await user.type(await prompt(), 'aws s3')
  const nav = within(screen.getByRole('navigation', { name: 'Principal' }))
  await user.click(nav.getByRole('button', { name: 'Inicio' }))
  expect(await screen.findByRole('heading', { level: 1 })).toHaveTextContent('Hola, Alex')
  await user.click(nav.getByRole('button', { name: 'Laboratorio' }))

  expect(await prompt()).toHaveValue('aws s3')
  expect(b.of('lab_start')).toHaveLength(1)
})

test('going to the class from the lab and back keeps the attempt', async () => {
  const user = userEvent.setup()
  const b = backend({ lab_start: lab() })
  renderLab()
  await user.type(await prompt(), 'aws s3')
  await user.click(screen.getByRole('button', { name: 'Ver explicación' }))

  expect(await screen.findByRole('heading', { level: 1 })).toHaveTextContent('Buckets y objetos')
  expect(screen.queryByRole('button', { name: 'Empezar práctica' })).toBeNull()
  await user.click(screen.getByRole('button', { name: 'Volver al laboratorio' }))

  expect(await prompt()).toHaveValue('aws s3')
  expect(b.of('lab_start')).toHaveLength(1)
})
