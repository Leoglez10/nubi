// Profile colors (AVATAR_COLORS in src/main.rs) and the radio group that picks one.
import { useId } from 'react'
import './ColorChoice.css'

export const AVATAR_COLORS: [string, string][] = [
  ['#BFE0FB', 'Azul cielo'],
  ['#FFE2B8', 'Atardecer'],
  ['#CFF2DF', 'Menta'],
  ['#E7DBFF', 'Lavanda'],
  ['#FFFFFF', 'Blanco'],
]

/** A color from the list, or the first one when a profile carries an unknown color. */
export const knownColor = (color: string) =>
  AVATAR_COLORS.find(([c]) => c.toLowerCase() === color.toLowerCase())?.[0] ?? AVATAR_COLORS[0][0]

export function ColorChoice({ value, onChange }: { value: string; onChange: (color: string) => void }) {
  const name = useId()
  return (
    <fieldset className="field color-choice">
      <legend className="field-label">Color</legend>
      <div className="swatches">
        {AVATAR_COLORS.map(([color, label]) => (
          <label key={color} className="swatch" style={{ background: color }} title={label}>
            <input type="radio" name={name} value={color} checked={value === color}
              onChange={() => onChange(color)} aria-label={label} />
          </label>
        ))}
      </div>
    </fieldset>
  )
}
