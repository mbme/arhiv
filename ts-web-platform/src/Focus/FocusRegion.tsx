import { noop } from '@v/utils'
import { useHotkeysMemo } from '@v/web-utils'
import * as React from 'react'
import {
  FocusManagerMode,
  FocusManager,
  FocusManagerContext,
} from './FocusContext'

const FocusRegionStyle = {
  display: 'contents',
}

function createFocusRegion(ref: React.MutableRefObject<HTMLDivElement | null>, context: FocusManager) {
  interface IProps {
    children: React.ReactNode
  }

  return function FocusRegion({ children }: IProps) {
    return (
      <FocusManagerContext.Provider value={context}>
        <div ref={ref} style={FocusRegionStyle}>
          {children}
        </div>
      </FocusManagerContext.Provider>
    )
  }
}

export function useFocusRegion(mode: FocusManagerMode) {
  const ref = React.useRef<HTMLDivElement>(null)
  const [context] = React.useState(() => new FocusManager(mode))

  React.useEffect(() => {
    if (!ref.current) {
      return noop
    }

    const el = ref.current

    const onMouseEnter = () => {
      context.activate()
    }
    const onMouseLeave = () => {
      context.deactivate()
    }
    el.addEventListener('mouseenter', onMouseEnter)
    el.addEventListener('mouseleave', onMouseLeave)

    return () => {
      el.removeEventListener('mouseenter', onMouseEnter)
      el.removeEventListener('mouseleave', onMouseLeave)
    }
  }, [ref.current])

  useHotkeysMemo(() => [
    {
      code: mode === 'row' ? 'KeyL' : 'KeyJ',
      action() {
        context.selectNext()
      },
    },
    {
      code: mode === 'row' ? 'KeyH' : 'KeyK',
      action() {
        context.selectPrevious()
      },
    },
    {
      code: 'Enter',
      action() {
        context.activateSelected()
      },
    },
  ], [context, mode])

  /* const parentContext = FocusManagerContext.use()
   */
  // FIXME on mouse enter, is focused
  // FIXME register region ref, register in parentContext

  const FocusRegion = React.useMemo(() => createFocusRegion(ref, context), [context])

  return {
    context,
    FocusRegion,
  }
}
