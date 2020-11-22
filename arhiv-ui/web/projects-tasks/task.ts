import { IDocument } from '../api'
import { DocumentDataDescription } from '../data-description'

const TASK_COMPLEXITY = ['Unknown' , 'Small' , 'Medium' , 'Large' , 'Epic'] as const
const TASK_STATUS = ['Inbox' , 'Todo' , 'Later' , 'InProgress' , 'Paused' , 'Done' , 'Cancelled'] as const

export const TaskDataDescription: DocumentDataDescription<ITaskProps> = {
  'title': { type: 'string' },
  'description': { type: 'markup-string' },
  'complexity': { type: 'enum', values: TASK_COMPLEXITY },
  'status': { type: 'enum', values: TASK_STATUS },
  'projectId': { type: 'reference' },
}

export interface ITaskProps {
  title: string
  description: string
  complexity: typeof TASK_COMPLEXITY[number],
  status: typeof TASK_STATUS[number],
  projectId: string
}

export type Task = IDocument<'task', ITaskProps>
