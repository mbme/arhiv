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
  Spacer,
} from '@v/web-platform'
import { pathMatcher as pm } from '@v/utils'
import { Header } from './Header'
import { Catalog } from './Catalog'
import { Card } from './Card'
import { CardEditorContainer } from './CardEditor'
import { NotFound } from './parts'

export function App() {
  return (
    <RouterProvider hashBased>
      <StylishProvider>
        <HotkeysResolverProvider>
          <OverlayRenderer>
            <Column
              width="500px"
              maxWidth="100%"
              alignX="stretch"
              mx="auto"
              bgColor="bg2"
              height="100%"
            >
              <Header />

              <Spacer height="large" />

              <Routes onNotFound={() => NotFound}>
                {[
                  [pm`/`, () => <Catalog />],
                  [pm`/new`, () => <CardEditorContainer />],
                  [pm`/${'id'}`, ({ id }) => <Card id={id} />],
                  [pm`/${'id'}/edit`, ({ id }) => <CardEditorContainer id={id} />],
                ]}
              </Routes>
            </Column>
          </OverlayRenderer>
        </HotkeysResolverProvider>
      </StylishProvider>
    </RouterProvider>
  )
}
