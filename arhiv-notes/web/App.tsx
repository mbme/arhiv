import * as React from 'react'
import {
  HotkeysResolverProvider,
  RouterProvider,
  Routes,
} from '@v/web-utils'
import {
  StylishProvider,
  OverlayRenderer,
  Column,
} from '@v/web-platform'
import { pathMatcher as pm } from '@v/utils'
import { Catalog } from './Catalog'
import { Card } from './Card'
import { CardEditorContainer } from './CardEditor'
import { NotFoundBlock } from './parts'
import { Url } from './Url'

export function App() {
  return (
    <RouterProvider hashBased>
      <StylishProvider>
        <HotkeysResolverProvider>
          <OverlayRenderer>
            <Column
              minWidth="30rem"
              maxWidth="40rem"
              alignX="stretch"
              height="100%"
            >
              <Routes
                onNotFound={() => <NotFoundBlock>View not found</NotFoundBlock>}
              >
                {[
                  [pm`/`, () => <Catalog />],
                  [pm`/new`, () => <CardEditorContainer />],
                  [pm`/${'id'}`, ({ id }) => <Card id={id} />],
                  [pm`/${'id'}/edit`, ({ id }) => <CardEditorContainer id={id} />],
                ]}
              </Routes>
            </Column>

            {process.env.NODE_ENV === 'development' && (
              <Url />
            )}
          </OverlayRenderer>
        </HotkeysResolverProvider>
      </StylishProvider>
    </RouterProvider>
  )
}
