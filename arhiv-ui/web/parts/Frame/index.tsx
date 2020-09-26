import * as React from 'react'
import {
  Box,
  StyleArg,
  FocusRegion,
} from '@v/web-platform'
import { Navigation } from './Navigation'
import { Content } from './Content'
import { Actions } from './Actions'

const $container: StyleArg = {
  display: 'grid',
  gridTemplateColumns: '10rem auto 10rem 1fr',
  gridAutoFlow: 'column',
  height: '100vh',
  overflowY: 'hidden',
}

interface IProps {
  children: React.ReactNode
  actions: React.ReactNode
  $style?: StyleArg
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
          <Navigation />
        </FocusRegion>

        <FocusRegion
          name="Content"
          mode="column"
        >
          <Content $style={$style}>
            {children}
          </Content>
        </FocusRegion>

        <FocusRegion
          name="Actions"
          mode="column"
        >
          <Actions actions={actions} />
        </FocusRegion>
      </Box>
    </FocusRegion>
  )
}
