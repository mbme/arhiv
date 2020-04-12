import * as React from 'react'
import { removeMut, getLastEl } from '@v/utils'
import { IKeybinding, isMatchingEvent } from './utils'
import { HotkeysResolverContext, IHotkeyResolver } from './context'

interface IProps {
  children: React.ReactNode
}

export function HotkeysResolverProvider({ children }: IProps) {
  const allHotkeys = React.useRef<Array<IKeybinding[]>>([])
  const [documents, setDocuments] = React.useState([document])

  const resolver = React.useMemo<IHotkeyResolver>(() => ({
    add(hotkeys) {
      allHotkeys.current.push(hotkeys)
    },
    remove(hotkeys) {
      removeMut(allHotkeys.current, hotkeys)
    },
    addDocument(document: Document) {
      setDocuments(documents => [...documents, document])
    },
    removeDocument(document: Document) {
      setDocuments(documents => documents.filter(item => item !== document))
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

    for (const document of documents) {
      document.addEventListener('keydown', handler)
    }

    return () => {
      for (const document of documents) {
        document.removeEventListener('keydown', handler)
      }
    }
  }, [documents])

  return (
    <HotkeysResolverContext.Provider value={resolver}>
      {children}
    </HotkeysResolverContext.Provider>
  )
}
