/* eslint-disable max-len */
import * as React from 'react'
import { Example } from '../Example'
import { Box } from '../Box'
import { Text } from '../Text'
import { Grid, Row } from '../Layout'

const colors = [
  '--color-primary',
  '--color-secondary',
  '--color-text',
  '--color-text-light',
  '--color-heading',
  '--color-link',
  '--color-bg0',
  '--color-bg-overlay',
]

export function StyleExamples() {
  return (
    <Example section title="Theme">
      <Example title="Colors">
        <Grid>
          {colors.map(value => (
            <Row
              key={value}
              height="3rem"
            >
              <Box
                width="10rem"
                display="flex"
                justifyContent="flex-end"
                mr="0.5rem"
              >
                {value}
              </Box>

              <Box
                width="3rem"
                height="3rem"
                backgroundColor={`var(${value})`}
                border="1px solid purple"
              />
            </Row>
          ))}
        </Grid>
      </Example>

      <Example title="General Font">
        <Text>
          Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.
        </Text>
      </Example>

      <Example title="Monospace font">
        <Text mono>
          {'fn fac_with_acc(n: u128, acc: u128) -> Rec<u128> { ... }'}
        </Text>
      </Example>
    </Example>
  )
}
