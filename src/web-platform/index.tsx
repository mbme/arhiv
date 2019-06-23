import * as React from 'react'
import { Button, examples as buttonExamples } from './Button'
import { Select, examples as selectExamples } from './Select'
import { Input, examples as inputExamples } from './Input'
import { FilterInput } from './FilterInput'
import { Textarea, examples as textareaExamples } from './Textarea'
import { Icon, examples as iconExamples } from './Icon'
import { AttachFileButton } from './AttachFileButton'
import theme, { examples as themeExamples } from './theme'
import {
  Box,
  FlexRow,
} from './Box'
import {
  Overlay,
  OverlayRenderer,
  ProgressLocker,
  Modal,
  ConfirmationDialog,
  confirmationDialogExamples,
} from './Overlay'

export { globalStyles } from './global-styles'

export { Link } from '~/web-router'
export { CleanLink } from './CleanLink'
export { Image } from './Image'

export {
  theme,
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
  FlexRow,
}

const renderExamples = (title: string, examples: { [description: string]: JSX.Element }) => (
  <Box $mb="xlarge">
    <h1>{title}</h1>
    {Object.entries(examples).map(([description, el], i) => (
      <Box key={i} $mb="medium">
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
