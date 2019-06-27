import * as React from 'react'

export * from './style'

import { Button, examples as buttonExamples } from './Button'
import { Select, examples as selectExamples } from './Select'
import { Input, examples as inputExamples } from './Input'
import { FilterInput } from './FilterInput'
import { Textarea, examples as textareaExamples } from './Textarea'
import { Icon, examples as iconExamples } from './Icon'
import { AttachFileButton } from './AttachFileButton'
import { examples as themeExamples } from './style'
import { Box } from './Box'
import {
  Overlay,
  OverlayRenderer,
  ProgressLocker,
  Modal,
  ConfirmationDialog,
  confirmationDialogExamples,
} from './Overlay'

export { Link } from '~/web-router'
export { CleanLink } from './CleanLink'
export { Image } from './Image'
export {
  Row,
  Column,
  Spacer,
} from './Layout'

export {
  Button,
  Select,
  Input,
  Textarea,
  FilterInput,
  Icon,
  Overlay,
  OverlayRenderer,
  ProgressLocker,
  Modal,
  ConfirmationDialog,
  AttachFileButton,
  Box,
}

const renderExamples = (title: string, examples: { [description: string]: JSX.Element }) => (
  <Box mb="xlarge">
    <h1>{title}</h1>
    {Object.entries(examples).map(([description, el], i) => (
      <Box key={i} mb="medium">
        {description && (
          <h2>{description}</h2>
        )}
        {el}
      </Box>
    ))}
  </Box>
)

export function Library() {
  return (
    <div>
      {renderExamples('Theme', themeExamples)}
      {renderExamples('Buttons', buttonExamples)}
      {renderExamples('Select', selectExamples)}
      {renderExamples('Input', inputExamples)}
      {renderExamples('Textarea', textareaExamples)}
      {renderExamples('Icons', iconExamples)}
      {renderExamples('ConfirmationDialog', confirmationDialogExamples)}
    </div>
  )
}
