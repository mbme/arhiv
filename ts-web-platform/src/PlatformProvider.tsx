import * as React from 'react'

import { HotkeysResolverProvider, RouterProvider } from '@v/web-utils'
import { StylishProvider } from './core'
import { OverlayRenderer } from './Modal'
import { FocusProvider, FocusRegion } from './Focus'

interface IProps {
  children: React.ReactNode
}

export function PlatformProvider({ children }: IProps) {
  return (
    <RouterProvider hashBased>
      <StylishProvider>
        <HotkeysResolverProvider>
          <FocusProvider>
            <OverlayRenderer>
              <FocusRegion
                name="Root"
                mode="column"
              >
                {children}
              </FocusRegion>
            </OverlayRenderer>
          </FocusProvider>
        </HotkeysResolverProvider>
      </StylishProvider>
    </RouterProvider>
  )
}
