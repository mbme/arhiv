import * as React from 'react'

import { HotkeysResolverProvider, RouterProvider } from '@v/web-utils'
import { StylishProvider } from './core'
import { OverlayRenderer } from './Modal'

interface IProps {
  children: React.ReactNode
}

export function PlatformProvider({ children }: IProps) {
  return (
    <RouterProvider hashBased>
      <StylishProvider>
        <HotkeysResolverProvider>
          <OverlayRenderer>
            {children}
          </OverlayRenderer>
        </HotkeysResolverProvider>
      </StylishProvider>
    </RouterProvider>
  )
}
