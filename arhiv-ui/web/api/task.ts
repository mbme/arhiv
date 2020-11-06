import { IDocument } from './types'

export interface ITaskProps {
  title: string
  description: string
  complexity: 'Unknown' | 'Small' | 'Medium' | 'Large' | 'Epic',
  status: 'Inbox' | 'Todo' | 'Later' | 'InProgress' | 'Paused' | 'Done' | 'Cancelled',
  projectId: string
}

export type Task = IDocument<'task', ITaskProps>
