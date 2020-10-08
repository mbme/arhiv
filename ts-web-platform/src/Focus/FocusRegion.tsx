import * as React from 'react'
import {
  HotkeysResolverContext,
  IKeybinding,
} from '@v/web-utils'
import { noop } from '@v/utils'
import { FocusManagerContext } from './context'
import { FocusManager, FocusManagerMode } from './focus-manager'
import { FocusStackContext } from './FocusProvider'

const FocusRegionStyle = {
  display: 'contents',
}

function useDefaultKeybindings(focusManager?: FocusManager) {
  const hotkeysResolver = HotkeysResolverContext.use()

  React.useEffect(() => {
    if (!focusManager) {
      return noop
    }

    const hotkeys: IKeybinding[] = [
      {
        code: focusManager.mode === 'row' ? 'KeyL' : 'KeyJ',
        action() {
          focusManager.selectNextChild()
          focusManager.scrollSelectedChildIntoView()
        },
      },
      {
        code: focusManager.mode === 'row' ? 'KeyH' : 'KeyK',
        action() {
          focusManager.selectPreviousChild()
          focusManager.scrollSelectedChildIntoView()
        },
      },
      {
        code: 'Enter',
        action(e) {
          e.preventDefault()
          focusManager.activateSelectedChild()
        },
      },
    ]

    return focusManager.enabled$.value$.subscribe({
      next(isEnabled) {
        if (isEnabled) {
          hotkeysResolver.add(hotkeys)
        } else {
          hotkeysResolver.remove(hotkeys)
        }
      },
    })
  }, [focusManager, hotkeysResolver])
}

interface IProps {
  name: string
  mode: FocusManagerMode
  children: React.ReactNode
}

export function FocusRegion({ children, mode, name }: IProps) {
  const parentFocusManager = React.useContext(FocusManagerContext)
  const focusStack = React.useContext(FocusStackContext)

  const ref = React.useRef<HTMLDivElement>(null)

  const [focusManager, setFocusManager] = React.useState<FocusManager | undefined>(undefined)
  useDefaultKeybindings(focusManager)

  React.useEffect(() => {
    if (!ref.current) {
      throw new Error('node must be available')
    }

    const prefixedName = parentFocusManager ? `${parentFocusManager.name}>${name}` : name
    const newFocusManager = new FocusManager(ref.current, mode, prefixedName)
    setFocusManager(newFocusManager)

    return () => {
      newFocusManager.destroy()
    }
  }, [])

  React.useEffect(() => {
    if (!focusManager) {
      return noop
    }

    // non-root focus region
    if (parentFocusManager) {
      return parentFocusManager.registerChild(focusManager)
    }

    // root focus region

    if (!focusStack) {
      throw new Error('FocusStack must be available')
    }

    return focusStack.add(focusManager)
  }, [focusManager])

  return (
    <div ref={ref} style={FocusRegionStyle}>
      <FocusManagerContext.Provider value={focusManager}>
        {children}
      </FocusManagerContext.Provider>
    </div>
  )
}
