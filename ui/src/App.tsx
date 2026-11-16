import { useEffect, useState } from 'react'
import { api, classes, type Content } from './api'

export default function App() {
  const [content, setContent] = useState<Content | null>(null)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    api.course().then(setContent, (e) => setError(String(e)))
  }, [])

  if (error) return <p role="alert">No se pudo cargar el curso: {error}</p>
  if (!content) return <p>Cargando…</p>
  return (
    <main>
      <h1>Nubi</h1>
      <ol>
        {classes(content).map(({ node, teaching }) => (
          <li key={node.id}>Clase {teaching.number}: {teaching.title}</li>
        ))}
      </ol>
    </main>
  )
}
