import * as React from 'react'
import { HotkeysResolverContext } from '@v/web-utils'
import { FocusManagerContext } from './context'
import { FocusManager, FocusManagerMode } from './focus-manager'
import { noop } from '@v/utils'
import { FocusStackContext } from './FocusProvider'

const FocusRegionStyle = {
  display: 'contents',
}

function useDefaultKeybindings(focusManager: FocusManager) {
  const hotkeysResolver = HotkeysResolverContext.use()

  React.useEffect(() => {
    const hotkeys = [
      {
        code: focusManager.mode === 'row' ? 'KeyL' : 'KeyJ',
        action() {
          focusManager.selectNextNode()
          focusManager.scrollSelectedNodeIntoView()
        },
      },
      {
        code: focusManager.mode === 'row' ? 'KeyH' : 'KeyK',
        action() {
          focusManager.selectPreviousNode()
          focusManager.scrollSelectedNodeIntoView()
        },
      },
      {
        code: 'Enter',
        action() {
          focusManager.activateSelectedNode()
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

  const [focusManager] = React.useState(() => new FocusManager(mode, `${parentFocusManager?.name || ''}>${name}`))
  useDefaultKeybindings(focusManager)

  // Root focus region
  React.useEffect(() => {
    if (parentFocusManager) {
      return noop
    }

    if (!focusStack) {
      throw new Error('FocusStack must be available')
    }

    return focusStack.add(focusManager)
  }, [])

  // Non-root focus region
  React.useEffect(() => {
    if (!parentFocusManager) {
      return noop
    }

    const childCount = ref.current?.childElementCount || 0
    if (childCount !== 1) {
      throw new Error(`FocusRegion must have a single DOM child, got ${childCount}`)
    }

    const node = ref.current?.firstChild as HTMLElement

    const unregister = parentFocusManager.registerNode(node)

    // enable region when selected
    const unsubIsSelected = parentFocusManager.isNodeSelected$(node).subscribe({
      next(isSelected: boolean) {
        if (isSelected) {
          focusManager.enable()
        } else {
          focusManager.disable()
        }
      }
    })

    // disable region if parent region disabled
    const unsubIsEnabled = parentFocusManager.enabled$.value$.subscribe({
      next(isEnabled) {
        if (!isEnabled) {
          focusManager.disable()
        }
      }
    })

    // enable region when hovered
    const onMouseEnter = () => {
      focusManager.enable()
    }
    const onMouseLeave = () => {
      focusManager.disable()
    }
    node.addEventListener('mouseenter', onMouseEnter)
    node.addEventListener('mouseleave', onMouseLeave)

    return () => {
      node.removeEventListener('mouseenter', onMouseEnter)
      node.removeEventListener('mouseleave', onMouseLeave)

      unsubIsSelected()
      unsubIsEnabled()

      unregister()
    }
  }, [])

  return (
    <FocusManagerContext.Provider value={focusManager}>
      <div ref={ref} style={FocusRegionStyle}>
        {children}
      </div>
    </FocusManagerContext.Provider>
  )
}
