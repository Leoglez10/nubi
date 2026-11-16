// The intro (src/view/intro.rs): Nubi rises over the clouds, the logo and the local-cloud
// checks appear one after another, then "Entrar". Static when motion is reduced.
import { useEffect, useState, type CSSProperties } from 'react'
import { api } from '../api'
import { Nubi } from '../art'
import { useApp } from '../store'
import { isReady } from './Setup'
import './Intro.css'

export function Intro() {
  const { state, go, reducedMotion } = useApp()
  const report = state.setup?.report
  const [saved, setSaved] = useState<boolean | null>(null)

  useEffect(() => {
    api.profiles().then(() => setSaved(true), () => setSaved(false))
  }, [])

  const checks: [string, boolean | null][] = [
    ['Docker encontrado', report ? report.docker_running : null],
    ['Tu nube local (AWS Emulator)', report ? report.emulator : null],
    ['Tu progreso está guardado en esta computadora', saved],
  ]
  // Leaves the intro: to setup when something is missing, else to the profiles (State::enter).
  const enter = () => go(state.setup && isReady(state.setup) ? 'profiles' : 'setup')

  return (
    <section className={`intro screen-full${reducedMotion ? ' still' : ''}`}>
      <Cloud className="sky-cloud c1" />
      <Cloud className="sky-cloud c2" />
      <Cloud className="sky-cloud c3" />
      <Cloud className="sky-cloud c4" />
      <div className="intro-center">
        <div className="intro-nubi"><Nubi size={200} /></div>
        <h1 id="page-title" tabIndex={-1} className="intro-logo">Nubi</h1>
        <p className="intro-tagline">Lab Cloud · Aprende AWS en tu propia nube</p>
        <ul className="intro-checks" aria-label="Tu computadora">
          {checks.map(([label, ok], i) => (
            <li key={label} style={{ '--i': i } as CSSProperties}>
              <span className={`mark ${ok == null ? 'pending' : ok ? 'ok' : 'missing'}`} aria-hidden="true">
                {ok == null ? '…' : ok ? '✓' : '!'}
              </span>
              <span>{label}</span>
              <span className="visually-hidden">{ok == null ? ': revisando' : ok ? ': listo' : ': falta'}</span>
            </li>
          ))}
        </ul>
        {state.setup && (
          <button type="button" className="btn btn-primary intro-enter" onClick={enter}>
            {isReady(state.setup) ? 'Entrar' : 'Preparar mi computadora'}
          </button>
        )}
      </div>
    </section>
  )
}

function Cloud({ className }: { className: string }) {
  return (
    <svg className={className} viewBox="0 0 120 96" aria-hidden="true" focusable="false">
      <path d="M30 80h62a22 22 0 0 0 2-44 30 30 0 0 0-57-6A24 24 0 0 0 30 80z" fill="#fff" />
    </svg>
  )
}
