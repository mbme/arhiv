import { IDocument } from './types'
import { IDocumentDataDescription } from '../data-description'

export const ProjectDataDescription: IDocumentDataDescription = {
  'title': { type: 'string' },
  'description': { type: 'markup-string' },
}

export interface IProjectProps {
  title: string,
  description: string,
}

export type Project = IDocument<'project', IProjectProps>
