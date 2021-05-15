import * as React from 'react'
import { IKeybinding } from './resolver'
import { HotkeysResolverContext } from './context'

export function useHotkeys(priority: number, hotkeys: IKeybinding[]) {
  const resolver = HotkeysResolverContext.use()

  React.useEffect(() => {
    resolver.add(priority, hotkeys)

    return () => {
      resolver.remove(hotkeys)
    }
  }, [priority, hotkeys])
}
