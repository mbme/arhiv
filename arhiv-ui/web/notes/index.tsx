import * as React from 'react'
import { pathMatcher as pm } from '@v/utils'
import { Route } from '@v/web-utils'
import { Catalog } from './Catalog'
import { Card } from './Card'
import { CardEditorContainer } from './CardEditor'

export const routes: Route<any>[] = [
  [pm`/notes`, () => <Catalog />],
  [pm`/notes/new`, () => <CardEditorContainer />],
  [pm`/notes/${'id'}`, ({ id }) => <Card id={id} />],
  [pm`/notes/${'id'}/edit`, ({ id }) => <CardEditorContainer id={id} />],
]
