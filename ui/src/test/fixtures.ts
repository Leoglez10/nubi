import type { Content, ProgressView } from '../api'

const teaching = (number: number, title: string, lab: string) =>
  ({ number, title, lab, explanation: 'Explicación', visual: 'Visual', example: 'Ejemplo', review: 'Repaso', diagram: null })

export const content: Content = {
  course: { units: [{ id: 's3', title: 'Fundamentos de S3', nodes: ['s3-buckets', 'lab-primer-bucket', 's3-objetos', 'lab-objetos'] }] },
  nodes: {
    's3-buckets': { id: 's3-buckets', title: 'Buckets', kind: 'leccion', exercises: [], objectives: [], files: [], teaching: teaching(1, 'Buckets y objetos', 'lab-primer-bucket') },
    'lab-primer-bucket': { id: 'lab-primer-bucket', title: 'Lab 1', kind: 'lab', teaching: null, exercises: [], objectives: [], files: [] },
    's3-objetos': { id: 's3-objetos', title: 'Objetos', kind: 'leccion', exercises: [], objectives: [], files: [], teaching: teaching(2, 'Claves y prefijos', 'lab-objetos') },
    'lab-objetos': { id: 'lab-objetos', title: 'Lab 2', kind: 'lab', teaching: null, exercises: [], objectives: [], files: [] },
  },
}

export function me(stars: Record<string, number> = {}): ProgressView {
  const nodes = Object.fromEntries(Object.entries(stars).map(([id, s]) => [id, { best_stars: s, best_errors: 0, attempts: 1 }]))
  return {
    profile: { id: 'alex', name: 'Alex', color: '#BFE0FB', reduced_motion: false },
    progress: { nodes },
    level: { number: 1, name: 'Despegando', from: 0, next_at: 3 },
    stops: [],
    saving: true,
  }
}
