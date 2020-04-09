import * as React from 'react'
import { removeMut, getLastEl } from '@v/utils'
import { IKeybinding, isMatchingEvent } from './utils'
import { HotkeysResolverContext, IHotkeyResolver } from './context'

interface IProps {
  children: React.ReactNode
}

export function HotkeysResolverProvider({ children }: IProps) {
  const allHotkeys = React.useRef<Array<IKeybinding[]>>([])

  const resolver = React.useMemo<IHotkeyResolver>(() => ({
    add(hotkeys) {
      allHotkeys.current.push(hotkeys)
    },
    remove(hotkeys) {
      removeMut(allHotkeys.current, hotkeys)
    },
  }), [])

  React.useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      const hotkeys = getLastEl(allHotkeys.current)

      if (!hotkeys) {
        return
      }

      const keybinding = hotkeys.find(item => isMatchingEvent(item, e))

      if (keybinding) {
        keybinding.action(e)
      }
    }

    document.addEventListener('keydown', handler)

    return () => {
      document.removeEventListener('keydown', handler)
    }
  }, [])

  return (
    <HotkeysResolverContext.Provider value={resolver}>
      {children}
    </HotkeysResolverContext.Provider>
  )
}
