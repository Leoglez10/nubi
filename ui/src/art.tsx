// Vector art shared by every screen: line icons, Nubi and profile avatars.

const ICONS = {
  home: 'M3 10.5L12 3l9 7.5V20a1 1 0 0 1-1 1h-5v-6H9v6H4a1 1 0 0 1-1-1z',
  book: 'M4 5a2 2 0 0 1 2-2h13v16H6a2 2 0 0 0-2 2zM4 5v16M8 7h7',
  check: 'M9 11l3 3 8-8M20 12v7a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11',
  term: 'M4 17l6-5-6-5M12 19h8',
  gear: 'M12 15a3 3 0 1 0 0-6 3 3 0 0 0 0 6zM19.4 15a1.7 1.7 0 0 0 .3 1.8l.1.1a2 2 0 1 1-2.8 2.8l-.1-.1a1.7 1.7 0 0 0-1.8-.3 1.7 1.7 0 0 0-1 1.5V21a2 2 0 1 1-4 0v-.1a1.7 1.7 0 0 0-1.1-1.5 1.7 1.7 0 0 0-1.8.3l-.1.1a2 2 0 1 1-2.8-2.8l.1-.1a1.7 1.7 0 0 0 .3-1.8 1.7 1.7 0 0 0-1.5-1H3a2 2 0 1 1 0-4h.1a1.7 1.7 0 0 0 1.5-1.1 1.7 1.7 0 0 0-.3-1.8l-.1-.1a2 2 0 1 1 2.8-2.8l.1.1a1.7 1.7 0 0 0 1.8.3H9a1.7 1.7 0 0 0 1-1.5V3a2 2 0 1 1 4 0v.1a1.7 1.7 0 0 0 1 1.5 1.7 1.7 0 0 0 1.8-.3l.1-.1a2 2 0 1 1 2.8 2.8l-.1.1a1.7 1.7 0 0 0-.3 1.8V9a1.7 1.7 0 0 0 1.5 1H21a2 2 0 1 1 0 4h-.1a1.7 1.7 0 0 0-1.5 1z',
  users: 'M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2M9 11a4 4 0 1 0 0-8 4 4 0 0 0 0 8zM22 21v-2a4 4 0 0 0-3-3.9M16 3.1a4 4 0 0 1 0 7.8',
  logout: 'M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4M16 17l5-5-5-5M21 12H9',
  chevron: 'M6 9l6 6 6-6',
} as const

export type IconName = keyof typeof ICONS

export function Icon({ name, size = 18 }: { name: IconName; size?: number }) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.75"
      strokeLinecap="round" strokeLinejoin="round" aria-hidden="true" focusable="false">
      <path d={ICONS[name]} />
    </svg>
  )
}

export type Mood = 'neutral' | 'happy' | 'worried'
const MOUTHS: Record<Mood, string> = { neutral: 'M54 64q8 7 16 0', happy: 'M51 62q11 11 22 0', worried: 'M54 68q8-7 16 0' }

/** Nubi, the mascot. Decorative unless given a label. */
export function Nubi({ mood = 'neutral', size = 40, label }: { mood?: Mood; size?: number; label?: string }) {
  return (
    <svg width={size} height={size * 0.8} viewBox="0 0 120 96" role={label ? 'img' : undefined}
      aria-label={label} aria-hidden={label ? undefined : true}>
      <path d="M30 80h62a22 22 0 0 0 2-44 30 30 0 0 0-57-6A24 24 0 0 0 30 80z" fill="#fff" stroke="#101828" strokeWidth="3.5" strokeLinejoin="round" />
      <circle cx="50" cy="54" r="5" fill="#101828" />
      <circle cx="74" cy="54" r="5" fill="#101828" />
      <circle cx="52" cy="52" r="1.6" fill="#fff" />
      <circle cx="76" cy="52" r="1.6" fill="#fff" />
      <path d={MOUTHS[mood]} fill="none" stroke="#101828" strokeWidth="3" strokeLinecap="round" />
      <circle cx="42" cy="64" r="4" fill="#fda29b" />
      <circle cx="82" cy="64" r="4" fill="#fda29b" />
    </svg>
  )
}

/** A profile's initial on its color. */
export function Avatar({ name, color, size = 32 }: { name: string; color: string; size?: number }) {
  return (
    <span aria-hidden="true" style={{
      display: 'inline-grid', placeItems: 'center', flex: 'none', inlineSize: size, blockSize: size,
      borderRadius: '50%', background: color, color: 'var(--text)', fontWeight: 700, fontSize: size * 0.42,
    }}>
      {name.trim().charAt(0).toUpperCase() || '?'}
    </span>
  )
}
