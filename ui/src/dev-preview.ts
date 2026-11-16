// Dev only: in a plain browser (no Tauri) answer commands with sample data, so layouts can be
// checked at any width with `npm run dev`. Never bundled into the app (see main.tsx).
import { mockIPC } from '@tauri-apps/api/mocks'
import { content, me } from './test/fixtures'

export function previewWithSampleData() {
  const profile = me({ 's3-buckets': 3 })
  const replies: Record<string, unknown> = {
    course: content,
    profiles: [{ profile: profile.profile, stars: 3 }],
    select_profile: profile,
    probe: {
      report: { docker: true, docker_running: true, aws_cli: true, emulator: true },
      needs: [],
      ready: true,
      os: 'mac',
    },
    fix: 'Vista previa: aquí se iniciaría el emulador.',
    create_profile: profile,
    update_profile: profile,
  }
  mockIPC((cmd) => replies[cmd] ?? null)
}
