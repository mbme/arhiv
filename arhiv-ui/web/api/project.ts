import { IDocument } from './types'

export interface IProjectProps {
  title: string,
  description: string,
}

export type Project = IDocument<'project', IProjectProps>
