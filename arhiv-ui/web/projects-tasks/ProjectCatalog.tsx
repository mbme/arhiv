import * as React from 'react'
import { RouterContext } from '@v/web-utils'
import { Catalog } from '../parts'
import { Project, ProjectDataDescription } from './project'
import { Matcher } from '../api'

export function ProjectCatalog() {
  const router = RouterContext.use()

  const getMatchers = (filter: string): Matcher[] => [
    { Type: 'project' },
    { Data: { selector: '$.name', pattern: filter } },
  ]

  const onAdd = () => router.push('/projects/new')
  const onActivate = (document: Project) => router.push(`/projects/${document.id}`)

  return (
    <Catalog
      title="Projects Catalog"
      dataDescription={ProjectDataDescription}
      getMatchers={getMatchers}
      onAdd={onAdd}
      onActivate={onActivate}
    />
  )
}
