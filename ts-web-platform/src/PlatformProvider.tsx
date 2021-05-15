import * as React from 'react'

import { HotkeysResolverProvider } from './utils'
import { StylishProvider } from './core'
import { OverlayRenderer } from './Modal'
import { RouterProvider } from './router'

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
