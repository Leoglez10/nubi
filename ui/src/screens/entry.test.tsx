import { act, render, screen, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { mockIPC, clearMocks } from '@tauri-apps/api/mocks'
import { afterEach, beforeAll, expect, test } from 'vitest'
import type { ProgressView, SetupView } from '../api'
import { Screens } from '../screens'
import { AppProvider, useApp, type AppState } from '../store'
import { content, me } from '../test/fixtures'

afterEach(clearMocks)

// jsdom has <dialog> but not its modal methods.
beforeAll(() => {
  const proto = HTMLDialogElement.prototype
  proto.showModal ??= function (this: HTMLDialogElement) { this.open = true }
  proto.close ??= function (this: HTMLDialogElement, value?: string) {
    this.open = false
    if (value != null) this.returnValue = value
    this.dispatchEvent(new Event('close'))
  }
})

const READY: SetupView = {
  report: { docker: true, docker_running: true, aws_cli: true, emulator: true },
  needs: [], ready: true, os: 'mac',
}
const NOTHING: SetupView = {
  report: { docker: false, docker_running: false, aws_cli: false, emulator: false },
  needs: [
    { need: 'docker', action: { type: 'run', value: { program: 'brew', args: ['install', '--cask', 'docker'] } } },
    { need: 'aws_cli', action: { type: 'run', value: { program: 'brew', args: ['install', 'awscli'] } } },
    { need: 'emulator', action: { type: 'start_emulator' } },
  ],
  ready: false, os: 'mac',
}
const NO_EMULATOR: SetupView = {
  report: { docker: true, docker_running: true, aws_cli: true, emulator: false },
  needs: [{ need: 'emulator', action: { type: 'start_emulator' } }],
  ready: false, os: 'mac',
}

/** Records every command sent, answering from `replies`. */
function backend(replies: Record<string, unknown>) {
  const calls: { cmd: string; args: unknown }[] = []
  mockIPC((cmd, args) => {
    calls.push({ cmd, args })
    const r = replies[cmd]
    if (r instanceof Error) throw r.message
    return typeof r === 'function' ? r(args) : r
  })
  return calls
}

function Where() {
  const { state } = useApp()
  return <output data-testid="where">{state.screen}:{state.me?.profile.name ?? '-'}</output>
}

function renderAt(patch: Partial<AppState>) {
  const initial: AppState = { content, screen: 'intro', stopId: null, setup: null, me: null, accountOpen: false, ...patch }
  return render(<AppProvider initial={initial}><Screens /><Where /></AppProvider>)
}
const where = () => screen.getByTestId('where')
const need = (name: string) => screen.getByText(name, { selector: 'strong' }).closest('li') as HTMLElement

// --- Intro ---

test('the intro goes to setup when something is missing', async () => {
  const user = userEvent.setup()
  backend({ profiles: [] })
  renderAt({ setup: NO_EMULATOR })
  await user.click(screen.getByRole('button', { name: 'Preparar mi computadora' }))
  expect(where()).toHaveTextContent('setup')
})

test('the intro goes to the profiles when everything is ready', async () => {
  const user = userEvent.setup()
  backend({ profiles: [] })
  renderAt({ setup: READY })
  await user.click(screen.getByRole('button', { name: 'Entrar' }))
  expect(where()).toHaveTextContent('profiles')
})

// --- Setup ---

test('setup lists each missing tool with its fix, and the emulator waits for Docker', () => {
  backend({})
  renderAt({ screen: 'setup', setup: NOTHING })
  expect(within(need('Docker')).getByRole('button', { name: 'Instalar Docker' })).toBeInTheDocument()
  expect(within(need('AWS CLI')).getByRole('button', { name: 'Instalar AWS CLI' })).toBeInTheDocument()
  expect(within(need('AWS Emulator')).getByText('Tu nube local: AWS en tu computadora, sin cuenta ni factura.')).toBeInTheDocument()
  expect(within(need('AWS Emulator')).queryByRole('button')).toBeNull()
  expect(screen.queryByText('Docker encendido')).toBeNull()
})

test('a missing emulator shows its warning and its start button', () => {
  backend({})
  renderAt({ screen: 'setup', setup: NO_EMULATOR })
  expect(need('AWS Emulator')).toHaveClass('missing')
  expect(within(need('AWS Emulator')).getByRole('button', { name: 'Iniciar emulador' })).toBeInTheDocument()
  expect(within(need('Docker')).getByText('Listo')).toBeInTheDocument()
})

test('a fix that answers with a note shows it', async () => {
  const user = userEvent.setup()
  const note = 'Abrimos la página oficial de descarga. Cuando termines de instalar, pulsa «Comprobar de nuevo».'
  const calls = backend({ fix: note })
  const setup: SetupView = { ...NOTHING, needs: [{ need: 'docker', action: { type: 'open_url', value: 'https://www.docker.com/' } }] }
  renderAt({ screen: 'setup', setup })
  await user.click(screen.getByRole('button', { name: 'Descargar Docker' }))
  expect(await screen.findByText(note)).toBeInTheDocument()
  expect(calls).toContainEqual({ cmd: 'fix', args: { need: 'docker', onProgress: expect.objectContaining({ onmessage: expect.any(Function) }) } })
})

test('setup shows CLI output while a fix is still running and preserves it on failure', async () => {
  const user = userEvent.setup()
  let fail!: (message: string) => void
  const pending = new Promise((_resolve, reject) => { fail = reject })
  const calls = backend({ fix: () => pending })
  renderAt({ screen: 'setup', setup: NO_EMULATOR })
  const start = screen.getByRole('button', { name: 'Iniciar emulador' })
  await user.click(start)
  const { onProgress } = calls.find(({ cmd }) => cmd === 'fix')?.args as {
    onProgress?: { onmessage: (message: { kind: string; text: string }) => void }
  }
  onProgress?.onmessage({ kind: 'stage', text: 'Construyendo la imagen del emulador' })
  onProgress?.onmessage({ kind: 'stderr', text: '#4 downloading' })
  const terminal = await screen.findByRole('region', { name: 'Salida de la preparación' })
  expect(terminal).toHaveTextContent('Construyendo la imagen del emulador')
  expect(terminal).toHaveTextContent('#4 downloading')
  expect(start).toBeDisabled()
  fail('Docker no pudo completar la descarga')
  expect(await screen.findByText('Docker no pudo completar la descarga')).toBeInTheDocument()
  expect(terminal).toHaveTextContent('#4 downloading')
  expect(start).toBeEnabled()
})

test('rechecking streams bounded plaintext without announcing raw output', async () => {
  const user = userEvent.setup()
  const calls = backend({ probe: () => new Promise(() => {}) })
  renderAt({ screen: 'setup', setup: NO_EMULATOR })
  await user.click(screen.getByRole('button', { name: 'Comprobar de nuevo' }))
  const { onProgress } = calls.find(({ cmd }) => cmd === 'probe')!.args as {
    onProgress: { onmessage: (event: { kind: string; text: string }) => void }
  }
  act(() => {
    onProgress.onmessage({ kind: 'stage', text: 'Revisando Docker' })
    onProgress.onmessage({ kind: 'stdout', text: `\x1b[31m${'x'.repeat(21000)}<b>fin</b>\x1b[0m` })
  })
  const terminal = screen.getByRole('region', { name: 'Salida de la preparación' })
  expect(terminal).toHaveAttribute('tabindex', '0')
  expect(terminal.textContent!.length).toBeLessThanOrEqual(20000)
  expect(terminal).toHaveTextContent('<b>fin</b>')
  expect(terminal.querySelector('b')).toBeNull()
  expect(terminal.textContent).not.toContain('\x1b')
  expect(screen.getAllByRole('status').some((el) => el.textContent === 'Revisando Docker')).toBe(true)
})

test('a finished fix probes again and moves on once ready', async () => {
  const user = userEvent.setup()
  backend({ fix: null, probe: READY, profiles: [] })
  renderAt({ screen: 'setup', setup: NO_EMULATOR })
  await user.click(screen.getByRole('button', { name: 'Iniciar emulador' }))
  expect(await screen.findByRole('heading', { name: '¿Quién va a aprender hoy?' })).toBeInTheDocument()
})

// --- Profiles ---

test('an empty name is refused and nothing is created', async () => {
  const user = userEvent.setup()
  const calls = backend({ profiles: [] })
  renderAt({ screen: 'profiles' })
  await user.type(await screen.findByRole('textbox', { name: 'Nombre' }), '   ')
  await user.click(screen.getByRole('button', { name: 'Crear y empezar' }))
  expect(screen.getByRole('textbox', { name: 'Nombre' })).toHaveAttribute('aria-invalid', 'true')
  expect(calls.map((c) => c.cmd)).not.toContain('create_profile')
})

test('a valid new profile is created with its color and signs in', async () => {
  const user = userEvent.setup()
  const calls = backend({ profiles: [], create_profile: me() })
  renderAt({ screen: 'profiles' })
  await user.type(await screen.findByRole('textbox', { name: 'Nombre' }), '  Alex ')
  await user.click(screen.getByRole('radio', { name: 'Menta' }))
  await user.click(screen.getByRole('button', { name: 'Crear y empezar' }))
  expect(calls).toContainEqual({ cmd: 'create_profile', args: { name: 'Alex', color: '#CFF2DF' } })
  expect(where()).toHaveTextContent('home:Alex')
})

test('picking a profile goes home signed in', async () => {
  const user = userEvent.setup()
  const alex = me()
  const calls = backend({ profiles: [{ profile: alex.profile, stars: 7 }], select_profile: alex })
  renderAt({ screen: 'profiles' })
  const card = await screen.findByRole('button', { name: /Alex/ })
  expect(card).toHaveTextContent('Nivel 2 · Explorador')
  expect(card).toHaveTextContent('★ 7 estrellas')
  await user.click(card)
  expect(calls).toContainEqual({ cmd: 'select_profile', args: { id: 'alex' } })
  expect(where()).toHaveTextContent('home:Alex')
})

// --- Ajustes ---

function signedIn(patch: Partial<ProgressView['profile']> = {}): ProgressView {
  const view = me()
  return { ...view, profile: { ...view.profile, ...patch } }
}

test('Ajustes saves a renamed profile', async () => {
  const user = userEvent.setup()
  const calls = backend({ update_profile: (a: { profile: unknown }) => ({ ...signedIn(), profile: a.profile }) })
  renderAt({ screen: 'settings', me: signedIn() })
  const name = screen.getByRole('textbox', { name: 'Nombre' })
  await user.clear(name)
  await user.type(name, 'Sam')
  await user.click(screen.getByRole('button', { name: 'Guardar cambios' }))
  expect(calls).toContainEqual({ cmd: 'update_profile', args: { profile: { ...signedIn().profile, name: 'Sam' } } })
  expect(where()).toHaveTextContent('settings:Sam')
})

test('deleting asks first; cancel sends nothing', async () => {
  const user = userEvent.setup()
  const calls = backend({ delete_profile: null })
  renderAt({ screen: 'settings', me: signedIn() })
  await user.click(screen.getByRole('button', { name: 'Eliminar perfil' }))
  const dialog = screen.getByRole('dialog', { name: '¿Eliminar a Alex?' })
  await user.click(within(dialog).getByRole('button', { name: 'Cancelar' }))
  expect(calls.map((c) => c.cmd)).not.toContain('delete_profile')
  expect(where()).toHaveTextContent('settings:Alex')
})

test('confirming the delete removes the profile and returns to the profiles', async () => {
  const user = userEvent.setup()
  const calls = backend({ delete_profile: null, profiles: [] })
  renderAt({ screen: 'settings', me: signedIn() })
  await user.click(screen.getByRole('button', { name: 'Eliminar perfil' }))
  await user.click(within(screen.getByRole('dialog')).getByRole('button', { name: 'Sí, eliminar' }))
  expect(calls).toContainEqual({ cmd: 'delete_profile', args: { id: 'alex' } })
  expect(where()).toHaveTextContent('profiles:-')
})

test('the motion switch saves reduced motion to the profile', async () => {
  const user = userEvent.setup()
  const calls = backend({ update_profile: (a: { profile: unknown }) => ({ ...signedIn(), profile: a.profile }) })
  renderAt({ screen: 'settings', me: signedIn() })
  const toggle = screen.getByRole('switch', { name: 'Reducir movimiento' })
  expect(toggle).not.toBeChecked()
  await user.click(toggle)
  expect(calls).toContainEqual({ cmd: 'update_profile', args: { profile: { ...signedIn().profile, reduced_motion: true } } })
  expect(screen.getByRole('switch', { name: 'Reducir movimiento' })).toBeChecked()
})
