import * as React from 'react'
import { pathMatcher as pm } from '@v/utils'
import { Route } from '@v/web-utils'
import { ProjectCatalog } from './ProjectCatalog'

export const routes: Route<any>[] = [
  [pm`/projects`, () => <ProjectCatalog />],
  [pm`/projects/new`, () => null],
  [pm`/projects/tasks/${'id'}`, ({ id }) => null],
  [pm`/projects/tasks/${'id'}/edit`, ({ id }) => null],
  [pm`/projects/${'id'}`, ({ id }) => null],
  [pm`/projects/${'id'}/edit`, ({ id }) => null],
  [pm`/projects/${'id'}/tasks/new`, ({ id }) => null],
]
