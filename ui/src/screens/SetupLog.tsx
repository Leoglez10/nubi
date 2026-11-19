import { useEffect, useRef } from 'react'
import { useApp } from '../store'
import './Setup.css'

/** Output stays readable and keyboard-scrollable; only stage changes are announced. */
export function SetupLog() {
  const { state } = useApp()
  const output = useRef<HTMLDivElement>(null)
  const follow = useRef(true)
  useEffect(() => {
    if (follow.current && output.current) output.current.scrollTop = output.current.scrollHeight
  }, [state.setupLog])

  return (
    <>
      <p className="visually-hidden" role="status">{state.setupStage}</p>
      {state.setupLog && (
        <div
          ref={output}
          className="setup-log"
          role="region"
          aria-label="Salida de la preparación"
          tabIndex={0}
          onScroll={(event) => {
            const el = event.currentTarget
            follow.current = el.scrollHeight - el.scrollTop - el.clientHeight < 24
          }}
        >
          <pre>{state.setupLog}</pre>
        </div>
      )}
    </>
  )
}
