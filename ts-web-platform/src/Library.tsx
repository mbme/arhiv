import * as React from 'react'

import { StylishProvider } from './core'
import { OverlayRenderer } from './Overlay'
import { ButtonExamples } from './Button.examples'
import { SelectExamples } from './Select.examples'
import { ToggleExamples } from './Toggle.examples'
import { InputExamples } from './Input.examples'
import { TextareaExamples } from './Textarea.examples'
import { IconExamples } from './Icon.examples'
import { StyleExamples } from './core/examples'
import { ConfirmationDialogExamples } from './Overlay/ConfirmationDialog.examples'

export function Library() {
  return (
    <StylishProvider>
      <OverlayRenderer>
        <StyleExamples />

        <IconExamples />

        <ButtonExamples />

        <SelectExamples />

        <ToggleExamples />

        <InputExamples />

        <TextareaExamples />

        <ConfirmationDialogExamples />
      </OverlayRenderer>
    </StylishProvider>
  )
}
