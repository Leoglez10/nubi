import type { Content, Exercise, Objective, ProgressView } from '../api'

const teaching = (number: number, title: string, lab: string) =>
  ({ number, title, lab, explanation: 'Explicación', visual: 'Visual', example: 'Ejemplo', review: 'Repaso', diagram: null })

/** The first class's practice: one exercise of each type. */
export const exercises: Exercise[] = [
  { type: 'choice', prompt: '¿Qué guarda un bucket?', options: ['Objetos', 'Servidores', 'Usuarios'], answer: 0, hint: 'Piensa en archivos.' },
  { type: 'command', prompt: 'Lista tus buckets', answers: ['aws s3 ls'], hint: 'Empieza con aws s3.' },
  { type: 'order', prompt: 'Ordena los pasos para subir un archivo', steps: ['Crear el bucket', 'Subir el archivo', 'Listar el bucket'], hint: 'Primero hace falta el bucket.' },
  { type: 'match', prompt: 'Une cada concepto con su significado', pairs: [['Bucket', 'Contenedor'], ['Objeto', 'Archivo'], ['Clave', 'Nombre']], hint: 'La clave es el nombre completo.' },
]

/** The first lab's objectives, as in content/lessons/lab-primer-bucket.toml. */
export const objectives: Objective[] = [
  { text: 'Crea el bucket mi-primer-bucket', hint: 'aws s3api create-bucket --bucket mi-primer-bucket', check: { type: 'bucket_exists', bucket: 'mi-primer-bucket' } },
  { text: 'Activa el versionado', hint: 'aws s3api put-bucket-versioning --bucket mi-primer-bucket --versioning-configuration Status=Enabled', check: { type: 'bucket_versioning', bucket: 'mi-primer-bucket', status: 'Enabled' } },
  { text: 'Sube el archivo hola.txt', hint: 'aws s3 cp hola.txt s3://mi-primer-bucket/', check: { type: 'object_exists', bucket: 'mi-primer-bucket', key: 'hola.txt' } },
]

export const content: Content = {
  course: { units: [{ id: 's3', title: 'Fundamentos de S3', nodes: ['s3-buckets', 'lab-primer-bucket', 's3-objetos', 'lab-objetos'] }] },
  nodes: {
    's3-buckets': { id: 's3-buckets', title: 'Buckets', kind: 'leccion', exercises, objectives: [], files: [], teaching: teaching(1, 'Buckets y objetos', 'lab-primer-bucket') },
    'lab-primer-bucket': { id: 'lab-primer-bucket', title: 'Lab 1', kind: 'lab', teaching: null, exercises: [], objectives, files: [] },
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
