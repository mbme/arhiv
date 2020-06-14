/* eslint-disable @typescript-eslint/triple-slash-reference, @typescript-eslint/camelcase */
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
export type Note = IDocument<'note', INoteProps>

export interface IAttachment {
  readonly id: string
  readonly rev: number
  readonly createdAt: string
  readonly filename: string
}

interface IRPC {
  list(): Promise<Note[]>
  get_note(id: string): Promise<Note | null>
  put_note(note: Note): Promise<void>
  create_note(): Promise<Note>
  get_attachment(id: string): Promise<IAttachment | null>
  get_attachment_url(id: string): Promise<string | null>
}

export const API = window.RPC_PROXY as IRPC
