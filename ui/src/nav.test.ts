import { expect, test } from 'vitest'
import { crumb, currentClass, classOf } from './nav'
import { content } from './test/fixtures'

const progress = (ids: string[]) => ({ nodes: Object.fromEntries(ids.map((id) => [id, { best_stars: 3, best_errors: 0, attempts: 1 }])) })

test('the current class is the first whose lab is not done, else the last', () => {
  expect(currentClass(content)?.node.id).toBe('s3-buckets')
  expect(currentClass(content, progress(['s3-buckets']))?.node.id).toBe('s3-buckets')
  expect(currentClass(content, progress(['lab-primer-bucket']))?.node.id).toBe('s3-objetos')
  expect(currentClass(content, progress(['lab-primer-bucket', 'lab-objetos']))?.node.id).toBe('s3-objetos')
})

test('the crumb names the class and the part of it', () => {
  const c = classOf(content, 'lab-primer-bucket')
  expect(crumb('home', c)).toBe('Inicio')
  expect(crumb('class', c)).toBe('Clase 1 · Buckets y objetos')
  expect(crumb('lesson', c)).toBe('Clase 1 · Buckets y objetos · Práctica')
  expect(crumb('lab', c)).toBe('Clase 1 · Buckets y objetos · Laboratorio')
  expect(crumb('settings', c)).toBe('Ajustes')
  expect(crumb('class', undefined)).toBe('Inicio')
})
