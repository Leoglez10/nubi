// Guided setup: what is missing on this machine and the one-click fix for each.
import { useState } from 'react'
import { api, type Action, type Need, type SetupView } from '../api'
import { useApp } from '../store'
import { SetupLog } from './SetupLog'
import './Setup.css'

/** Everything is installed and running. A missing emulator always counts, even if `ready` says otherwise. */
export const isReady = (setup: SetupView) => setup.ready && Object.values(setup.report).every(Boolean)

const COPY: Record<Need, [string, string]> = {
  docker: ['Docker', 'Ejecuta tu nube local en un contenedor.'],
  docker_running: ['Docker encendido', 'Docker está instalado pero apagado.'],
  aws_cli: ['AWS CLI', 'Los comandos aws que practicarás en la terminal.'],
  emulator: ['AWS Emulator', 'Tu nube local: AWS en tu computadora, sin cuenta ni factura.'],
}

function fixLabel(need: Need, action: Action): string | null {
  if (action.type === 'start_emulator') return 'Iniciar emulador'
  if (action.type === 'manual') return null
  if (need === 'docker_running') return 'Abrir Docker'
  if (need === 'docker') return action.type === 'run' ? 'Instalar Docker' : 'Descargar Docker'
  if (need === 'aws_cli') return action.type === 'run' ? 'Instalar AWS CLI' : 'Cómo instalar AWS CLI'
  return 'Iniciar emulador'
}

/** What the worker said while a fix runs (Worker::fix in src/main.rs); opening a page is instant. */
function busyText(need: Need, action: Action): string | null {
  if (action.type === 'start_emulator') return 'Encendiendo tu nube local… la primera vez descarga el emulador (unos minutos).'
  if (action.type === 'run') return need === 'docker_running' ? 'Abriendo Docker…' : 'Instalando… esto puede tardar unos minutos.'
  return null
}

export function Setup() {
  const { state, set, go, setupProgress, clearSetupLog } = useApp()
  const [busy, setBusy] = useState<string | null>(null)
  const [working, setWorking] = useState(false)
  const [note, setNote] = useState<string | null>(null)
  const setup = state.setup

  const probe = async () => {
    const fresh = await api.probe(setupProgress)
    set({ setup: fresh })
    if (isReady(fresh)) go('profiles')
  }

  const recheck = async () => {
    setWorking(true)
    clearSetupLog()
    setNote(null)
    try {
      await probe()
    } catch (e) {
      setNote(String(e))
    } finally {
      setWorking(false)
    }
  }

  const fix = async (need: Need, action: Action) => {
    setWorking(true)
    clearSetupLog()
    setBusy(busyText(need, action))
    setNote(null)
    try {
      const said = await api.fix(need, setupProgress)
      if (said) setNote(said)
      else await probe()
    } catch (e) {
      setNote(String(e))
    } finally {
      setBusy(null)
      setWorking(false)
    }
  }

  return (
    <section className="setup screen-full">
      <div className="setup-card card">
        <header className="setup-head">
          <h1 id="page-title" tabIndex={-1}>Preparemos tu nube local</h1>
          <p className="setup-lead">Nubi necesita estas herramientas en tu computadora. Te ayudamos a instalarlas.</p>
        </header>
        {setup ? <Needs setup={setup} working={working} onFix={fix} /> : <p className="setup-status" role="status">Revisando tu computadora…</p>}
        <div aria-live="polite" className="setup-messages">
          {busy && <p className="setup-status">{busy}</p>}
          {note && <p className="note">{note}</p>}
        </div>
        <SetupLog />
        {!working && (
          <div className="setup-actions">
            <button type="button" className="btn" onClick={recheck}>Comprobar de nuevo</button>
          </div>
        )}
      </div>
    </section>
  )
}

function Needs({ setup, working, onFix }: {
  setup: SetupView
  working: boolean
  onFix: (need: Need, action: Action) => void
}) {
  const r = setup.report
  const missing = setup.needs.map((n) => n.need)
  const items: [Need, boolean][] = [
    ['docker', r.docker],
    ['docker_running', r.docker_running],
    ['aws_cli', r.aws_cli],
    ['emulator', r.emulator],
  ]
  return (
    <ul className="needs">
      {items
        // Installing Docker comes first: no "turn it on" step without it.
        .filter(([need]) => !(need === 'docker_running' && !r.docker))
        .map(([need, ok]) => {
          const [name, why] = COPY[need]
          const action = setup.needs.find((n) => n.need === need)?.action
          // Starting the emulator needs a running Docker first.
          const blocked = need === 'emulator' && (!r.docker || !r.docker_running || missing.includes('docker') || missing.includes('docker_running'))
          const label = action && fixLabel(need, action)
          return (
            <li key={need} className={`need ${ok ? 'ok' : 'missing'}`}>
              <span className="need-mark" aria-hidden="true">{ok ? '✓' : '!'}</span>
              <div className="need-text">
                <strong>{name}</strong>
                <span>{ok ? 'Listo' : why}</span>
              </div>
              {!ok && !blocked && action && (
                label
                  ? <button type="button" className="btn btn-primary" disabled={working} onClick={() => onFix(need, action)}>{label}</button>
                  : action.type === 'manual' && <span className="need-manual">{action.value}</span>
              )}
            </li>
          )
        })}
    </ul>
  )
}
