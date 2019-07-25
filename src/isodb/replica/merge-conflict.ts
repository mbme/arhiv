import { IDocument, IChangesetResult } from "../types";
import { IReplicaStorage } from "./replica-storage";

type ConflictType = 'both-updated'
  | 'local-updated-remote-deleted'
  | 'local-deleted-remote-updated'
  | 'both-deleted'
  | 'local-ref-deleted-attachment'
  | 'local-ref-deleted-document'
  | 'remote-ref-deleted-document'

interface IConflict<T extends IDocument> {
  conflictType: ConflictType
  base: T
  updated: T
  local: T
  resolved?: T
}

interface IChangesetMergeConflict<T extends IDocument> {
  changesetResult: IChangesetResult
  conflicts: IConflict<T>[]
}
