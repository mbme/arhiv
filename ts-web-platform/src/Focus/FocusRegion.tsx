import * as React from 'react'
import { noop } from '@v/utils'
import { FocusManagerContext } from './context'
import { FocusManager, FocusManagerMode } from './focus-manager'
import { FocusStackContext } from './FocusProvider'
import { Box } from '../Box'
import { StyleArg } from '../core'
import { useDefaultKeybindings } from './useDefaultKeybindings'

const $title: StyleArg = {
  position: 'absolute',
  top: 0,
  left: 0,
  right: 0,
  textAlign: 'center',
  textTransform: 'uppercase',
  bgColor: 'orange',
  height: '1.3rem',
}

const getStyles = (highlight?: boolean, withTitle?: boolean): StyleArg[] => [
  {
    position: 'relative',
  },
  highlight && {
    bgColor: 'var(--color-bg-highlight)',
  },
  withTitle && {
    paddingTop: '1.3rem',
  },
]

interface IProps {
  name: string
  mode: FocusManagerMode
  highlight?: boolean
  $style?: StyleArg
  title?: string
  children: React.ReactNode
}

export function FocusRegion({ children, mode, name, highlight, title, $style }: IProps) {
  const parentFocusManager = React.useContext(FocusManagerContext)
  const focusStack = React.useContext(FocusStackContext)

  const ref = React.useRef<HTMLDivElement>(null)

  const [focusManager, setFocusManager] = React.useState<FocusManager | undefined>(undefined)
  const [isEnabled, setIsEnabled] = React.useState(false)

  useDefaultKeybindings(focusManager)

  React.useEffect(() => {
    if (!ref.current) {
      throw new Error('node must be available')
    }

    const prefixedName = parentFocusManager ? `${parentFocusManager.name}>${name}` : name
    const newFocusManager = new FocusManager(ref.current, mode, prefixedName)
    setFocusManager(newFocusManager)

    const unsub = newFocusManager.enabled$.value$.subscribe({
      next: setIsEnabled
    })

    return () => {
      newFocusManager.destroy()
      unsub()
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
    <Box
      ref={ref}
      $styles={getStyles(isEnabled && highlight, !!title)}
      $style={$style}
    >
      {title && (
        <Box
          $style={$title}
        >
          {title}
        </Box>
      )}

      <FocusManagerContext.Provider value={focusManager}>
        {children}
      </FocusManagerContext.Provider>
    </Box>
  )
}
