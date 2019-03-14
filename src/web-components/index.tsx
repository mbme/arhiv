import * as React from 'react'
import { Button, examples as buttonExamples } from './Button'
import { Select, examples as selectExamples } from './Select'
import { Input, examples as inputExamples } from './Input'
import { FilterInput } from './FilterInput'
import { Textarea, examples as textareaExamples } from './Textarea'
import { Icon, examples as iconExamples } from './Icon'
import {
  Overlay,
  OverlayRenderer,
  ProgressLocker,
  Modal,
  ConfirmationDialog,
  confirmationDialogExamples,
} from './Overlay'

export { globalStyles } from './global-styles'
export { default as theme } from './theme'

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
}

const renderExamples = (title: string, examples: { [description: string]: JSX.Element }) => (
  <div>
    <h1>{title}</h1>
    {Object.entries(examples).map(([description, el], i) => (
      <div key={i}>
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
      {renderExamples('Buttons', buttonExamples)}
      {renderExamples('Select', selectExamples)}
      {renderExamples('Input', inputExamples)}
      {renderExamples('Textarea', textareaExamples)}
      {renderExamples('Icons', iconExamples)}
      {renderExamples('ConfirmationDialog', confirmationDialogExamples)}
    </div>
  )
}
