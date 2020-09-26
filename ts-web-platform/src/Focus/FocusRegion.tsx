import * as React from 'react'
import { noop } from '@v/utils'
import { HotkeysResolverContext } from '@v/web-utils'
import { FocusManagerContext } from './FocusContext'
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

    // automatically activate region when selected
    const unsub = parentFocusManager.isSelected$(node).subscribe({
      next(isSelected: boolean) {
        if (isSelected) {
          focusManager.activate()
        } else {
          focusManager.deactivate()
        }
      }
    })

    return () => {
      unregister()
      unsub()
    }
  }, [parentFocusManager])

  // activate region when hovered
  React.useEffect(() => {
    const node = ref.current?.firstChild as HTMLElement | undefined
    if (!node) {
      throw new Error('child node is missing')
    }

    const onMouseEnter = () => {
      focusManager.activate()
    }
    const onMouseLeave = () => {
      focusManager.deactivate()
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
          // FIXME scroll into view
          focusManager.selectNext()
        },
      },
      {
        code: mode === 'row' ? 'KeyH' : 'KeyK',
        action() {
          // FIXME scroll into view
          focusManager.selectPrevious()
        },
      },
      {
        code: 'Enter',
        action() {
          focusManager.activateSelected()
        },
      },
    ]

    return focusManager.active$.value$.subscribe({
      next(isActive) {
        if (isActive) {
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
