import { render, screen } from '@testing-library/react'
import { mockIPC, clearMocks } from '@tauri-apps/api/mocks'
import { afterEach, expect, test } from 'vitest'
import App from './App'
import type { Content } from './api'

afterEach(clearMocks)

const content: Content = {
  course: { units: [{ id: 's3', title: 'S3', nodes: ['s3-buckets', 'lab-primer-bucket'] }] },
  nodes: {
    's3-buckets': {
      id: 's3-buckets', title: 'Buckets', kind: 'leccion', exercises: [], objectives: [], files: [],
      teaching: { number: 1, title: 'Buckets y objetos', lab: 'lab-primer-bucket', explanation: '', visual: '', example: '', review: '', diagram: null },
    },
    'lab-primer-bucket': { id: 'lab-primer-bucket', title: 'Lab', kind: 'lab', teaching: null, exercises: [], objectives: [], files: [] },
  },
}

test('lists the classes the Rust side returns', async () => {
  mockIPC((cmd) => (cmd === 'course' ? content : undefined))
  render(<App />)
  expect(await screen.findByText('Clase 1: Buckets y objetos')).toBeInTheDocument()
})

test('says so when the course cannot be loaded', async () => {
  mockIPC(() => { throw new Error('roto') })
  render(<App />)
  expect(await screen.findByRole('alert')).toHaveTextContent('No se pudo cargar el curso')
})
