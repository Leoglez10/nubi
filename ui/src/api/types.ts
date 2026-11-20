// Mirrors the serde shape of `nubi::course` (src/course.rs).

export type NodeKind = 'leccion' | 'lab' | 'desafio'
export type Diagram = 'bucket_objects' | 'key_prefixes'

export interface Unit { id: string; title: string; nodes: string[] }

export interface Teaching {
  number: number
  title: string
  lab: string
  explanation: string
  visual: string
  example: string
  review: string
  diagram: Diagram | null
}

export type Exercise =
  | { type: 'choice'; prompt: string; options: string[]; answer: number; hint: string }
  | { type: 'command'; prompt: string; answers: string[]; hint: string }
  | { type: 'order'; prompt: string; steps: string[]; hint: string }
  | { type: 'match'; prompt: string; pairs: [string, string][]; hint: string }

export type Check =
  | { type: 'bucket_exists'; bucket: string }
  | { type: 'bucket_versioning'; bucket: string; status: string }
  | { type: 'object_exists'; bucket: string; key: string }
  | { type: 'dynamo_table' | 'dynamo_item' | 'dynamo_read' }

export interface Objective { text: string; hint: string; check: Check }
export interface LabFile { name: string; content: string }

export interface Node {
  id: string
  title: string
  kind: NodeKind
  teaching: Teaching | null
  exercises: Exercise[]
  objectives: Objective[]
  files: LabFile[]
}

export interface Content {
  course: { units: Unit[] }
  nodes: Record<string, Node>
}

/** Classes in map order, like `Content::classes` in Rust. */
export function classes(content: Content): { node: Node; teaching: Teaching }[] {
  return content.course.units
    .flatMap((u) => u.nodes)
    .map((id) => content.nodes[id])
    .filter((n): n is Node & { teaching: Teaching } => n?.teaching != null)
    .map((node) => ({ node, teaching: node.teaching }))
}

// --- Engine views (src-tauri/src/engine.rs) ---

export interface Profile { id: string; name: string; color: string; reduced_motion: boolean }
export interface ProfileCard { profile: Profile; stars: number }

export interface NodeResult { best_stars: number; best_errors: number; attempts: number }
export interface Progress { nodes: Record<string, NodeResult> }

export interface Level { number: number; name: string; from: number; next_at: number | null }

export type Stop = { type: 'done'; stars: number } | { type: 'current' } | { type: 'locked' }
export interface StopView { id: string; status: Stop }

export interface ProgressView {
  profile: Profile
  progress: Progress
  level: Level
  stops: StopView[]
  /** False when saved progress was unreadable: playing works, saving does not. */
  saving: boolean
}

export type Need = 'docker' | 'docker_running' | 'aws_cli' | 'emulator'
export type Action =
  | { type: 'run'; value: { program: string; args: string[] } }
  | { type: 'open_url'; value: string }
  | { type: 'start_emulator' }
  | { type: 'manual'; value: string }

export interface Report { docker: boolean; docker_running: boolean; aws_cli: boolean; emulator: boolean }
export interface SetupView {
  report: Report
  needs: { need: Need; action: Action }[]
  ready: boolean
  os: 'mac' | 'windows' | 'linux'
}

export type Answer =
  | { type: 'choice'; value: number }
  | { type: 'command'; value: string }
  | { type: 'order'; value: number[] }
  | { type: 'match'; value: [number, number][] }

export type Feedback = { type: 'right' } | { type: 'wrong'; hint: string }

export interface LessonView {
  id: string
  session: { index: number; total: number; errors: number; mistakes: string[]; feedback: Feedback | null }
  finished: boolean
  progress: number
  saved: ProgressView | null
}

export type LineKind = 'cmd' | 'out' | 'err' | 'ok' | 'info'
export interface LabView {
  id: string
  table?: string | null
  session: {
    lines: { kind: LineKind; text: string }[]
    done: boolean[]
    errors: number
    mistakes: { command: string; error: string }[]
  }
  complete: boolean
  saved: ProgressView | null
}
