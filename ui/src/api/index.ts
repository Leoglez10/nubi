import { Channel, invoke } from '@tauri-apps/api/core'
import type {
  Answer, Content, LabView, LessonView, Need, Profile, ProfileCard, ProgressView, SetupView,
} from './types'

export * from './types'

export type SetupProgress = { kind: 'stage' | 'stdout' | 'stderr'; text: string }

function progressChannel(onProgress: (event: SetupProgress) => void) {
  const channel = new Channel<SetupProgress>()
  channel.onmessage = onProgress
  return channel
}

/** The Rust commands in src-tauri/src/main.rs, typed. Errors reject with a Spanish message. */
export const api = {
  course: () => invoke<Content>('course'),

  probe: (onProgress: (event: SetupProgress) => void) => invoke<SetupView>('probe', { onProgress: progressChannel(onProgress) }),
  /** Resolves with a note to show, or null when done (probe again). */
  fix: (need: Need, onProgress: (event: SetupProgress) => void) => invoke<string | null>('fix', { need, onProgress: progressChannel(onProgress) }),

  profiles: () => invoke<ProfileCard[]>('profiles'),
  createProfile: (name: string, color: string) => invoke<ProgressView>('create_profile', { name, color }),
  selectProfile: (id: string) => invoke<ProgressView>('select_profile', { id }),
  updateProfile: (profile: Profile) => invoke<ProgressView>('update_profile', { profile }),
  deleteProfile: (id: string) => invoke<void>('delete_profile', { id }),
  signOut: () => invoke<void>('sign_out'),
  progress: () => invoke<ProgressView | null>('progress'),

  shuffled: (n: number) => invoke<number[]>('shuffled', { n }),
  lessonStart: (id: string) => invoke<LessonView>('lesson_start', { id }),
  lessonCheck: (answer: Answer) => invoke<LessonView>('lesson_check', { answer }),
  lessonRetry: () => invoke<LessonView>('lesson_retry'),
  lessonNext: () => invoke<LessonView>('lesson_next'),

  labStart: (id: string) => invoke<LabView>('lab_start', { id }),
  labSubmit: (line: string) => invoke<LabView>('lab_submit', { line }),
  labHistory: (older: boolean) => invoke<string | null>('lab_history', { older }),
}
