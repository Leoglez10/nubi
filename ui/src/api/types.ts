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
