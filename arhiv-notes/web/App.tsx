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
              mx="auto"
              bgColor="bg2"
              height="100%"
            >
              <Header />

              <Spacer height="large" />

              <Routes onNotFound={() => <NotFoundBlock>View not found</NotFoundBlock>}>
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
