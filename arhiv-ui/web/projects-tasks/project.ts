import { IDocument } from '../api'
import { DocumentDataDescription } from '../data-description'

export const ProjectDataDescription: DocumentDataDescription<IProjectProps> = {
  'title': { type: 'string' },
  'description': { type: 'markup-string' },
}

export interface IProjectProps {
  title: string
  description: string
}

export type Project = IDocument<'project', IProjectProps>
