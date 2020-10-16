import * as React from 'react'
import { noop } from '@v/utils'
import { FocusManagerContext } from './context'
import { FocusManager, FocusManagerMode } from './focus-manager'
import { FocusStackContext } from './FocusProvider'
import { Box } from '../Box'
import { StyleArg } from '../core'
import { useDefaultKeybindings } from './useDefaultKeybindings'

const getTitleStyles = (highlight?: boolean): StyleArg[] => [
  {
    position: 'absolute',
    top: 0,
    left: 0,
    right: 0,
    textAlign: 'center',
    textTransform: 'uppercase',
    fontFamily: 'var(--font-family-mono)',

    height: 'calc(var(--line-height) * 1rem)',

    bgColor: 'var(--color-bg-secondary)',
    transition: 'background-color 100ms linear',
  },

  highlight && {
    bgColor: 'var(--color-bg-primary)',
  },
]

const getContainerStyles = (withTitle?: boolean): StyleArg[] => [
  {
    position: 'relative',
  },
  withTitle && {
    paddingTop: 'calc(var(--line-height) * 1rem + 0.4rem)',
  },
]

interface IProps {
  name: string
  mode: FocusManagerMode
  highlight?: boolean
  $style?: StyleArg
  title?: string
  autoFocus?: boolean
  children: React.ReactNode
}

export function FocusRegion({ children, mode, name, highlight, title, autoFocus, $style }: IProps) {
  const parentFocusManager = React.useContext(FocusManagerContext)
  const focusStack = React.useContext(FocusStackContext)

  const ref = React.useRef<HTMLDivElement>(null)

  const [focusManager] = React.useState(() => {
    const prefixedName = parentFocusManager ? `${parentFocusManager.name}>${name}` : name

    const depth = parentFocusManager ? parentFocusManager.depth + 1 : 0

    return new FocusManager(mode, prefixedName, depth)
  })
  const [isEnabled, setIsEnabled] = React.useState(false)

  useDefaultKeybindings(focusManager)

  React.useEffect(() => {
    if (!ref.current) {
      throw new Error('node must be available')
    }

    focusManager.setDOMNode(ref.current)

    const unsub = focusManager.enabled$.value$.subscribe({
      next: setIsEnabled
    })

    return () => {
      focusManager.destroy()
      unsub()
    }
  }, [])

  React.useEffect(() => {
    if (!focusManager) {
      return noop
    }

    // non-root focus region
    if (parentFocusManager) {
      return parentFocusManager.registerChild(focusManager, autoFocus)
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
      $styles={getContainerStyles(!!title)}
      $style={$style}
    >
      {title && (
        <Box
          $styles={getTitleStyles(isEnabled && highlight)}
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
