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
  gridTemplateColumns: '1fr 2.2fr 1fr',
  gridAutoFlow: 'column',
  gridGap: '0.8rem',

  height: '100vh',
  overflowY: 'hidden',
  p: 'fine',

  maxWidth: '88rem',
  minWidth: '63rem',
  mx: 'auto',
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
          autoFocus
        >
          <Box
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
