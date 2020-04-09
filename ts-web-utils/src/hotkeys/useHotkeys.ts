import * as React from 'react'
import { IKeybinding } from './utils'
import { HotkeysResolverContext } from './context'

export function useHotkeys(hotkeys: IKeybinding[]) {
  const resolver = HotkeysResolverContext.use()

  React.useEffect(() => {
    resolver.add(hotkeys)

    return () => {
      resolver.remove(hotkeys)
    }
  }, [hotkeys])
}
