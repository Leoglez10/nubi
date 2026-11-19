import { render, screen, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { expect, test } from 'vitest'
import type { Content, Diagram, ProgressView } from '../api'
import { Screens } from '../screens'
import { AppProvider, type AppState } from '../store'
import { content, me } from '../test/fixtures'

function renderAt(patch: Partial<AppState>, profile: ProgressView | null = me(), course: Content = content) {
  const initial: AppState = { content: course, screen: 'home', stopId: null, setup: null, me: profile, accountOpen: false, ...patch }
  return render(<AppProvider initial={initial}><Screens /></AppProvider>)
}

/** The fixture course with diagrams on its two classes. */
function withDiagrams(): Content {
  const set = (id: string, diagram: Diagram) => ({ ...content.nodes[id], teaching: { ...content.nodes[id].teaching!, diagram } })
  return { ...content, nodes: { ...content.nodes, 's3-buckets': set('s3-buckets', 'bucket_objects'), 's3-objetos': set('s3-objetos', 'key_prefixes') } }
}

const title = () => screen.getByRole('heading', { level: 1 })

test('Inicio greets the profile and shows totals from saved progress', () => {
  renderAt({}, me({ 's3-buckets': 3, 'lab-primer-bucket': 2 }))
  expect(title()).toHaveTextContent('Hola, Alex')
  expect(screen.getByText('Llevas 1 de 2 clases del módulo de S3.')).toBeInTheDocument()
  const kpis = within(screen.getByRole('list', { name: 'Tu progreso' }))
  const kpi = (label: string) => kpis.getByText(label).closest('li')!
  expect(kpi('Clases completadas')).toHaveTextContent('1 / 2')
  expect(kpi('Estrellas')).toHaveTextContent('5de 12 posibles')
  expect(kpi('Prácticas')).toHaveTextContent('1de 2')
  expect(kpi('Nivel')).toHaveTextContent('1Despegando')
  expect(screen.getByText('Hoy toca claves y prefijos.')).toBeInTheDocument()
})

test('the continue card opens the current class', async () => {
  const user = userEvent.setup()
  renderAt({}, me({ 's3-buckets': 3, 'lab-primer-bucket': 2 }))
  expect(screen.getByText('CONTINÚA DONDE LO DEJASTE')).toBeInTheDocument()
  expect(screen.getByRole('heading', { name: 'Clase 2 · Claves y prefijos' })).toBeInTheDocument()
  await user.click(screen.getByRole('button', { name: 'Empezar clase' }))
  expect(title()).toHaveTextContent('Claves y prefijos')
  expect(screen.getByText('CLASE 2 · EXPLICACIÓN')).toBeInTheDocument()
})

test('a locked stop cannot be opened', () => {
  renderAt({}, me())
  const row = screen.getByRole('rowheader', { name: /Clase 2: Claves y prefijos/ }).closest('tr')!
  expect(row).toHaveTextContent('Bloqueada')
  expect(row).toHaveTextContent('Disponible al terminar la clase anterior')
  for (const name of ['Abrir la clase 2', 'Práctica de la clase 2', 'Laboratorio de la clase 2']) {
    expect(screen.getByRole('button', { name })).toBeDisabled()
  }
  expect(screen.getByRole('button', { name: 'Abrir la clase 1' })).toBeEnabled()
})

test('the lab button is locked until the practice is done', async () => {
  const user = userEvent.setup()
  const { unmount } = renderAt({}, me())
  expect(screen.getByRole('button', { name: 'Laboratorio de la clase 1' })).toBeDisabled()
  unmount()

  renderAt({}, me({ 's3-buckets': 2 }))
  await user.click(screen.getByRole('button', { name: 'Laboratorio de la clase 1' }))
  expect(screen.getByText('LABORATORIO')).toBeInTheDocument()
  expect(title()).toHaveTextContent('Lab 1')
})

test('the bucket pilot goes from the concept through a simulated upload to a retryable review', async () => {
  const user = userEvent.setup()
  renderAt({ screen: 'class', stopId: 's3-buckets' }, me(), withDiagrams())
  expect(screen.getByText('Escena 1 de 3')).toBeInTheDocument()
  await user.click(screen.getByRole('button', { name: 'Probar la subida' }))
  expect(screen.getByText('Escena 2 de 3')).toBeInTheDocument()
  expect(screen.queryByText('Objeto guardado en mi-bucket.')).not.toBeInTheDocument()
  expect(screen.queryByRole('button', { name: 'Ir al repaso' })).not.toBeInTheDocument()
  const upload = screen.getByRole('button', { name: 'Subir foto.jpg' })
  const diagram = screen.getByText('Bucket · mi-bucket')
  expect(diagram.compareDocumentPosition(upload) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy()
  expect(upload).toHaveClass('scene-primary-interactive')
  await user.tab()
  expect(upload).toHaveFocus()
  await user.keyboard('{Enter}')
  expect(screen.getByRole('status')).toHaveTextContent('Objeto guardado en mi-bucket.')
  expect(screen.getByText('mi-bucket', { selector: 'mark' })).toBeInTheDocument()
  expect(screen.getByText('foto.jpg', { selector: 'mark' })).toBeInTheDocument()
  const review = screen.getByRole('button', { name: 'Ir al repaso' })
  expect(review).toHaveClass('btn-primary')
  expect(review).not.toHaveClass('scene-primary-interactive')
  expect(review).toHaveFocus()
  await user.click(review)
  await user.click(screen.getByRole('button', { name: 'mi-bucket' }))
  expect(screen.getByRole('status')).toHaveAttribute('data-tone', 'hint')
  expect(screen.getByRole('status')).toHaveTextContent('Inténtalo de nuevo. Ese es el nombre del bucket')
  await user.click(screen.getByRole('button', { name: 'foto.jpg' }))
  expect(screen.getByRole('status')).toHaveAttribute('data-tone', 'success')
  expect(screen.getByRole('status')).toHaveTextContent('Correcto: foto.jpg es la clave del objeto.')
  await user.click(screen.getByRole('button', { name: 'Escena anterior' }))
  expect(screen.getByRole('status')).toHaveTextContent('Objeto guardado en mi-bucket.')
  expect(screen.getByRole('button', { name: 'Empezar práctica' })).toBeEnabled()
})

test('Claves y prefijos filters object keys and explains why the whole key matters', async () => {
  const user = userEvent.setup()
  renderAt({ screen: 'class', stopId: 's3-objetos' }, me({ 's3-buckets': 1, 'lab-primer-bucket': 1 }), withDiagrams())
  expect(title()).toHaveTextContent('Claves y prefijos')
  expect(screen.getByText('Escena 1 de 3')).toBeInTheDocument()
  const steps = within(screen.getByRole('complementary', { name: 'En esta clase' }))
  expect(steps.getAllByRole('listitem').map((li) => li.querySelector('.step-label')!.textContent))
    .toEqual(['Explicación', 'Ejemplo', 'Práctica', 'Laboratorio', 'Repaso'])
  expect(steps.getByText('Tip de Nubi')).toBeInTheDocument()
  expect(screen.getByText('PREFIJO')).toBeInTheDocument()
  expect(screen.getByText('informes/')).toBeInTheDocument()
  expect(screen.queryByText('Bucket: mi-bucket')).toBeNull()

  await user.click(screen.getByRole('button', { name: 'Explorar el bucket' }))
  const keys = () => within(screen.getByRole('list', { name: 'Claves del bucket archivo-nubi' }))
  expect(keys().getByText('borradores/enero.txt')).toBeInTheDocument()
  const filter = screen.getByRole('button', { name: 'Filtrar por informes/' })
  expect(filter).toHaveClass('scene-primary-interactive')
  await user.tab()
  expect(filter).toHaveFocus()
  await user.keyboard(' ')
  expect(keys().getByText('informes/enero.txt')).toBeInTheDocument()
  expect(keys().getByText('informes/febrero.txt')).toBeInTheDocument()
  expect(keys().queryByText('borradores/enero.txt')).not.toBeInTheDocument()
  expect(screen.getByRole('status')).toHaveTextContent('2 de 3 claves coinciden')
  expect(screen.getByText('aws s3 ls s3://archivo-nubi/informes/')).toBeInTheDocument()
  const review = screen.getByRole('button', { name: 'Ir al repaso' })
  expect(review).not.toHaveClass('scene-primary-interactive')
  await user.click(review)
  await user.click(screen.getByRole('button', { name: 'enero.txt' }))
  expect(screen.getByRole('status')).toHaveAttribute('data-tone', 'hint')
  await user.click(screen.getByRole('button', { name: 'informes/enero.txt' }))
  expect(screen.getByRole('status')).toHaveAttribute('data-tone', 'success')
  await user.click(screen.getByRole('button', { name: 'Escena anterior' }))
  expect(keys().queryByText('borradores/enero.txt')).not.toBeInTheDocument()
  await user.click(screen.getByRole('button', { name: 'Mostrar todas las claves' }))
  expect(keys().getByText('borradores/enero.txt')).toBeInTheDocument()
  await user.click(screen.getByRole('button', { name: 'Empezar práctica' }))
  expect(await screen.findByRole('heading', { level: 1 })).toHaveTextContent('Práctica')
})

test("Clase's practice button goes to the lesson and its lab button only once practiced", async () => {
  const user = userEvent.setup()
  const { unmount } = renderAt({ screen: 'class', stopId: 's3-buckets' }, me())
  expect(screen.getByRole('button', { name: 'Laboratorio: termina la práctica' })).toBeDisabled()
  expect(screen.queryByRole('button', { name: 'Ir al laboratorio' })).toBeNull()
  await user.click(screen.getByRole('button', { name: 'Empezar práctica' }))
  expect(await screen.findByRole('heading', { level: 1 })).toHaveTextContent('Práctica')
  unmount()

  renderAt({ screen: 'class', stopId: 's3-buckets' }, me({ 's3-buckets': 2 }))
  await user.click(screen.getByRole('button', { name: 'Ir al laboratorio' }))
  expect(screen.getByText('LABORATORIO')).toBeInTheDocument()
  expect(title()).toHaveTextContent('Lab 1')
})
