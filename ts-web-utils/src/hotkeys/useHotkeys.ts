import * as React from 'react'
import { IKeybinding } from './resolver'
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

export function useHotkeysMemo(cb: () => IKeybinding[], deps: any[] = []) {
  const hotkeys = React.useMemo(cb, deps)

  useHotkeys(hotkeys)
}
