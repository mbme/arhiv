import * as React from 'react'
import { isFunction } from '~/utils'

import { Box } from './Box'
import { Heading } from './Heading'

import { examples as buttonExamples } from './Button.examples'
import { examples as selectExamples } from './Select.examples'
import { examples as inputExamples } from './Input.examples'
import { examples as textareaExamples } from './Textarea.examples'
import { examples as iconExamples } from './Icon.examples'
import { examples as themeExamples } from './style'
import { confirmationDialogExamples } from './Overlay'

interface IProps {
  title: string
  examples: { [description: string]: JSX.Element | React.FC }
}

function Examples({ title, examples }: IProps) {
  return (
    <Box mb="xlarge">
      <Heading>{title}</Heading>

      {Object.entries(examples).map(([description, Example], i) => (
        <Box key={i} mb="medium">
          {description && (
            <Heading fontSize="medium">
              {description}
            </Heading>
          )}

          {isFunction(Example) ? <Example /> : Example}
        </Box>
      ))}
    </Box>
  )
}

export function Library() {
  return (
    <div>
      <Examples title="Theme" examples={themeExamples} />

      <Examples title="Icons" examples={iconExamples} />

      <Examples title="Buttons" examples={buttonExamples} />

      <Examples title="Select" examples={selectExamples} />

      <Examples title="Input" examples={inputExamples} />

      <Examples title="Textarea" examples={textareaExamples} />

      <Examples title="Confirmation dialog" examples={confirmationDialogExamples} />
    </div>
  )
}
