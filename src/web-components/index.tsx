import * as React from 'react'
import {
  style,
} from 'typestyle'
import { Button, examples as buttonExamples } from './Button'
import { Select, examples as selectExamples } from './Select'
import { Input, examples as inputExamples } from './Input'
import { FilterInput } from './FilterInput'
import { Textarea, examples as textareaExamples } from './Textarea'
import { Icon, examples as iconExamples } from './Icon'
import { AttachFileButton } from './AttachFileButton'
import theme, { examples as themeExamples } from './theme'
import {
  Overlay,
  OverlayRenderer,
  ProgressLocker,
  Modal,
  ConfirmationDialog,
  confirmationDialogExamples,
} from './Overlay'

export * from './styles'

export { globalStyles } from './global-styles'

export { Box } from './Box'

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
}

const renderExamples = (title: string, examples: { [description: string]: JSX.Element }) => (
  <div className={style({ marginBottom: '3rem' })}>
    <h1>{title}</h1>
    {Object.entries(examples).map(([description, el], i) => (
      <div key={i} className={style({ marginBottom: '1rem' })}>
        {description && (
          <h2>{description}</h2>
        )}
        {el}
      </div>
    ))}
  </div>
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
