import { act, render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import type { SetupView } from './api'
import { mockIPC, clearMocks } from '@tauri-apps/api/mocks'
import { afterEach, expect, test } from 'vitest'
import App from './App'
import { content } from './test/fixtures'

afterEach(clearMocks)

test('starts on the intro once the course loads', async () => {
  mockIPC((cmd) => (cmd === 'course' ? content : cmd === 'profiles' ? [] : undefined))
  render(<App />)
  expect(await screen.findByRole('heading', { name: 'Nubi' })).toBeInTheDocument()
})

test('the welcome screen keeps probe commands out of view but setup shows the transcript', async () => {
  const user = userEvent.setup()
  let channel: { onmessage: (event: { kind: string; text: string }) => void } | undefined
  let finishProbe!: (setup: SetupView) => void
  mockIPC((cmd, args) => {
    if (cmd === 'course') return content
    if (cmd === 'profiles') return []
    if (cmd === 'probe') {
      channel = (args as { onProgress: typeof channel }).onProgress
      return new Promise<SetupView>((resolve) => { finishProbe = resolve })
    }
  })
  render(<App />)
  await screen.findByRole('heading', { name: 'Nubi' })
  expect(channel).toBeDefined()
  act(() => channel!.onmessage({ kind: 'stage', text: '$ docker --version' }))
  expect(screen.queryByText('$ docker --version')).not.toBeInTheDocument()
  expect(screen.queryByRole('region', { name: 'Salida de la preparación' })).not.toBeInTheDocument()
  act(() => finishProbe({
    ready: false, os: 'mac',
    report: { docker: true, docker_running: true, aws_cli: true, emulator: false },
    needs: [{ need: 'emulator', action: { type: 'start_emulator' } }],
  }))
  await user.click(await screen.findByRole('button', { name: 'Preparar mi computadora' }))
  expect(screen.getByRole('region', { name: 'Salida de la preparación' })).toHaveTextContent('$ docker --version')
})

test('says so when the course cannot be loaded', async () => {
  mockIPC(() => { throw new Error('roto') })
  render(<App />)
  expect(await screen.findByRole('alert')).toHaveTextContent('No se pudo cargar el curso')
})
