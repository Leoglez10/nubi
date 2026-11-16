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
    probe: { report: { docker: true, docker_running: true, aws_cli: true, emulator: false }, needs: [], ready: false, os: 'mac' },
  }
  mockIPC((cmd) => replies[cmd] ?? null)
}
