import * as React from 'react'
import {
  Box,
  Column,
  StyleArg,
  Link,
  FocusRegion,
} from '@v/web-platform'

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
  return (
    <FocusRegion
      name="Frame"
      mode="row"
    >
      <Box
        as="section"
        $style={$container}
      >
        <FocusRegion
          name="Navigation"
          mode="column"
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
        </FocusRegion>

        <FocusRegion
          name="Content"
          mode="column"
        >
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
        </FocusRegion>

        <FocusRegion
          name="Actions"
          mode="column"
        >
          <Column
            pl="small"
            $style={$actionContainer}
          >
            {actions}
          </Column>
        </FocusRegion>
      </Box>
    </FocusRegion>
  )
}
