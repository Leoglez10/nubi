import { invoke } from '@tauri-apps/api/core'
import type { Content } from './types'

export * from './types'

/** The Rust commands in src-tauri/src/main.rs, typed. */
export const api = {
  course: () => invoke<Content>('course'),
}
