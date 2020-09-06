import * as React from 'react'

import { StylishProvider } from './core'
import { OverlayRenderer } from './Modal'
import { ButtonExamples } from './Button.examples'
import { SelectExamples } from './Form/Select.examples'
import { InputExamples } from './Form/Input.examples'
import { TextareaExamples } from './Form/Textarea.examples'
import { IconExamples } from './Icon.examples'
import { StyleExamples } from './core/examples'
import { ConfirmationDialogExamples } from './Modal/ConfirmationDialog.examples'

export function Library() {
  return (
    <StylishProvider>
      <OverlayRenderer>
        <StyleExamples />

        <IconExamples />

        <ButtonExamples />

        <SelectExamples />

        <InputExamples />

        <TextareaExamples />

        <ConfirmationDialogExamples />
      </OverlayRenderer>
    </StylishProvider>
  )
}
