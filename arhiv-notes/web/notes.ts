/* eslint-disable @typescript-eslint/triple-slash-reference */
/// <reference path="../../app-shell/src/rpc.d.ts" />

import { Obj } from '@v/utils'

interface IDocument<T extends string = string, P extends Obj = Obj> {
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

export type AttachmentLocation = { Url: string } | { File: string } | { Unknown: null }

interface IRPC {
  list(): Promise<Note[]>
  get_note(id: string): Promise<Note | null>
  put_note(note: Note): Promise<void>
  create_note(): Promise<Note>
  get_attachment(id: string): Promise<IAttachment | null>
  get_attachment_location(id: string): Promise<AttachmentLocation>
  pick_attachments(): Promise<IAttachment[]>
}

export const API = window.RPC_PROXY as IRPC
