import { render, screen, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { mockIPC, clearMocks } from '@tauri-apps/api/mocks'
import { afterEach, expect, test } from 'vitest'
import type { ProgressView } from '../api'
import { AppProvider, useApp, type AppState } from '../store'
import { content, me } from '../test/fixtures'
import { Shell } from './Shell'

afterEach(clearMocks)

function Probe() {
  const { state } = useApp()
  return <h1 id="page-title" tabIndex={-1}>{state.screen}:{state.stopId}</h1>
}

function renderShell(patch: Partial<AppState> = {}, profile: ProgressView | null = me()) {
  const initial: AppState = { content, screen: 'home', stopId: null, setup: null, me: profile, accountOpen: false, ...patch }
  return render(<AppProvider initial={initial}><Shell><Probe /></Shell></AppProvider>)
}

const nav = () => within(screen.getByRole('navigation', { name: 'Principal' }))

test('the sidebar opens the current class, its practice and, once practiced, its lab', async () => {
  const user = userEvent.setup()
  renderShell()
  expect(nav().getByRole('button', { name: 'Laboratorio' })).toBeDisabled()
  await user.click(nav().getByRole('button', { name: 'Práctica' }))
  expect(screen.getByRole('heading')).toHaveTextContent('lesson:s3-buckets')
  expect(screen.getByTestId('crumb')).toHaveTextContent('Clase 1 · Buckets y objetos · Práctica')
  expect(nav().getByRole('button', { name: 'Práctica' })).toHaveAttribute('aria-current', 'page')
})

test('the lab opens after the practice is done', async () => {
  const user = userEvent.setup()
  renderShell({}, me({ 's3-buckets': 2 }))
  await user.click(nav().getByRole('button', { name: 'Laboratorio' }))
  expect(screen.getByRole('heading')).toHaveTextContent('lab:lab-primer-bucket')
})

test('a new screen moves focus to its heading', async () => {
  const user = userEvent.setup()
  renderShell()
  await user.click(nav().getByRole('button', { name: 'Clase actual' }))
  expect(screen.getByRole('heading')).toHaveFocus()
})

test('the account menu holds settings, switching profile and signing out', async () => {
  const user = userEvent.setup()
  const calls: string[] = []
  mockIPC((cmd) => { calls.push(cmd) })
  renderShell()
  expect(nav().queryByRole('button', { name: 'Ajustes' })).toBeNull()
  await user.click(nav().getByRole('button', { name: /Alex/ }))
  expect(nav().getByRole('button', { name: 'Ajustes' })).toBeVisible()
  await user.click(nav().getByRole('button', { name: 'Cerrar sesión' }))
  expect(calls).toContain('sign_out')
  expect(screen.getByRole('heading')).toHaveTextContent('profiles:')
})

test('the top bar shows stars and level; the environment card shows what is missing', () => {
  renderShell({ setup: { report: { docker: true, docker_running: true, aws_cli: true, emulator: false }, needs: [], ready: false, os: 'mac' } },
    me({ 's3-buckets': 3, 'lab-primer-bucket': 2 }))
  expect(screen.getByText(/5 estrellas, nivel 1: Despegando/)).toBeInTheDocument()
  const env = within(screen.getByRole('region', { name: 'Entorno local' }))
  expect(env.getByText('Emulador S3: falta')).toBeInTheDocument()
  expect(env.getByText('Docker: listo')).toBeInTheDocument()
})
