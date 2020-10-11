import * as React from 'react'
import {
  Box,
  StyleArg,
  FocusRegion,
} from '@v/web-platform'
import { Navigation } from './Navigation'
import { Actions } from './Actions'

const $container: StyleArg = {
  display: 'grid',
  gridTemplateColumns: '10rem auto 10rem 1fr',
  gridAutoFlow: 'column',
  height: '100vh',
  overflowY: 'hidden',
}

const $content: StyleArg = {
  height: '100%',
  overflowY: 'auto',
}

interface IProps {
  children: React.ReactNode
  actions: React.ReactNode
  title: string
  $style?: StyleArg
}

export function Frame({ children, actions, title, $style }: IProps) {
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
          highlight
          title="Apps"
        >
          <Navigation />
        </FocusRegion>

        <FocusRegion
          name="Content"
          mode="column"
          highlight
          title={title}
          $style={$content}
        >
          <Box
            px="medium"
            width="40rem"
            height="100%"
            overflowY="auto"
            $style={$style}
          >
            {children}
          </Box>
        </FocusRegion>

        <FocusRegion
          name="Actions"
          mode="column"
          highlight
          title="Actions"
        >
          <Actions actions={actions} />
        </FocusRegion>
      </Box>
    </FocusRegion>
  )
}
