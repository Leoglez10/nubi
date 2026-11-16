import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import './index.css'
import App from './App.tsx'

if (import.meta.env.DEV && !('__TAURI_INTERNALS__' in window)) {
  const { previewWithSampleData } = await import('./dev-preview')
  previewWithSampleData()
}

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <App />
  </StrictMode>,
)
