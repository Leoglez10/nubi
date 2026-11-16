import { render, screen } from '@testing-library/react'
import { mockIPC, clearMocks } from '@tauri-apps/api/mocks'
import { afterEach, expect, test } from 'vitest'
import App from './App'
import { content } from './test/fixtures'

afterEach(clearMocks)

test('starts on the profile picker once the course loads', async () => {
  mockIPC((cmd) => (cmd === 'course' ? content : cmd === 'profiles' ? [] : undefined))
  render(<App />)
  expect(await screen.findByRole('heading', { name: '¿Quién aprende hoy?' })).toBeInTheDocument()
})

test('says so when the course cannot be loaded', async () => {
  mockIPC(() => { throw new Error('roto') })
  render(<App />)
  expect(await screen.findByRole('alert')).toHaveTextContent('No se pudo cargar el curso')
})
