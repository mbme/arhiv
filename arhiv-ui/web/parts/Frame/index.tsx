import * as React from 'react'
import {
  Box,
  Column,
  StyleArg,
  Link,
  useFocusRegion,
} from '@v/web-platform'
import { useHotkeysMemo } from '@v/web-utils'

const $container: StyleArg = {
  display: 'grid',
  gridTemplateColumns: '10rem auto 10rem 1fr',
  gridAutoFlow: 'column',
  height: '100vh',
  overflowY: 'hidden',
}

const $actionContainer: StyleArg = {
  '&>*': {
    my: 'small',
  },
}

interface IProps {
  children: React.ReactNode,
  actions: React.ReactNode,
  $style?: StyleArg,
}

export function Frame({ children, actions, $style }: IProps) {
  const {
    FocusRegion,
    setRef,
    context,
  } = useFocusRegion('column')

  useHotkeysMemo(() => {
    return [
      {
        code: 'KeyJ',
        action() {
          context.selectNext()
        },
      },
      {
        code: 'KeyK',
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
    ]
  }, [context])

  return (
    <FocusRegion>
      <Box
        as="section"
        $style={$container}
        innerRef={setRef}
      >
        <Column>
          <Box
            bgColor="var(--color-primary)"
            width="100%"
            display="flex"
            justifyContent="center"
          >
            <Link to={{ path: '/' }}>
              Notes
            </Link>
          </Box>

          <Link to={{ path: '/' }}>
            Contacts
          </Link>

          <Link to={{ path: '/' }}>
            Movies
          </Link>
        </Column>

        <Box
          px="medium"
          width="40rem"
          overflowY="auto"
          borderLeft="default"
          borderRight="default"
          $style={$style}
        >
          {children}
        </Box>

        <Column
          pl="small"
          $style={$actionContainer}
        >
          {actions}
        </Column>
      </Box>
    </FocusRegion>
  )
}
