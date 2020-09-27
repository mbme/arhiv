import * as React from 'react'
import { noop } from '@v/utils'
import { HotkeysResolverContext } from '@v/web-utils'
import { FocusManagerContext } from './context'
import { FocusManager, FocusManagerMode } from './focus-manager'

const FocusRegionStyle = {
  display: 'contents',
}

interface IProps {
  name: string
  mode: FocusManagerMode
  children: React.ReactNode
}

export function FocusRegion({ children, mode, name }: IProps) {
  const ref = React.useRef<HTMLDivElement>(null)
  const parentFocusManager = React.useContext(FocusManagerContext)
  const hotkeysResolver = HotkeysResolverContext.use()
  const [focusManager] = React.useState(() => new FocusManager(
    mode,
    parentFocusManager ? `${parentFocusManager.name}>${name}` : name,
  ))

  React.useEffect(() => {
    if (!parentFocusManager) {
      return noop
    }

    const node = ref.current?.firstChild as HTMLElement | undefined
    if (!node) {
      throw new Error('child node is missing')
    }

    const unregister = parentFocusManager.registerNode(node)

    // automatically enable region when selected
    const unsub = parentFocusManager.isNodeSelected$(node).subscribe({
      next(isSelected: boolean) {
        if (isSelected) {
          focusManager.enable()
        } else {
          focusManager.disable()
        }
      }
    })

    return () => {
      unregister()
      unsub()
    }
  }, [parentFocusManager])

  // enable region when hovered
  React.useEffect(() => {
    const childCount = ref.current?.childElementCount || 0
    if (childCount !== 1) {
      throw new Error(`FocusRegion must have a single child, got ${childCount}`)
    }

    const node = ref.current?.firstChild as HTMLElement

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
    }
  }, [])

  // install keybindings
  React.useEffect(() => {
    const hotkeys = [
      {
        code: mode === 'row' ? 'KeyL' : 'KeyJ',
        action() {
          focusManager.selectNextNode()
          focusManager.scrollSelectedNodeIntoView()
        },
      },
      {
        code: mode === 'row' ? 'KeyH' : 'KeyK',
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
  }, [focusManager, hotkeysResolver, mode])

  return (
    <FocusManagerContext.Provider value={focusManager}>
      <div ref={ref} style={FocusRegionStyle}>
        {children}
      </div>
    </FocusManagerContext.Provider>
  )
}
