// eslint-disable-next-line @typescript-eslint/triple-slash-reference
/// <reference path="../../app-shell/src/rpc.d.ts" />

interface IDocument<T extends string = string, P extends object = {}> {
  readonly id: string
  readonly rev: number
  readonly type: T
  readonly schemaVersion: number
  readonly createdAt: string
  readonly updatedAt: string
  readonly refs: readonly string[]
  readonly attachmentRefs: readonly string[]
  readonly archived: boolean
  readonly data: P
}

interface INoteProps {
  name: string,
  data: string,
}
type Note = IDocument<'note', INoteProps>

export function list(): Promise<Note[]> {
  return window.RPC.call('list')
}

export function getNote(id: string): Promise<Note | null> {
  return window.RPC.call('get_note', id)
}

export async function putNote(note: Note) {
  await window.RPC.call('get_note', note)
}
