import * as React from 'react'

import { HotkeysResolverProvider } from '@v/web-utils'
import { StylishProvider } from './core'
import { OverlayRenderer } from './Modal'
import { ButtonExamples } from './Button.examples'
import { SelectExamples } from './Form/Select.examples'
import { InputExamples } from './Form/Input.examples'
import { TextareaExamples } from './Form/Textarea.examples'
import { IconExamples } from './Icon.examples'
import { StyleExamples } from './core/examples'
import { ConfirmationDialogExamples } from './Modal/ConfirmationDialog.examples'
import { FocusRegion } from './Focus'

export function Library() {
  return (
    <StylishProvider>
      <HotkeysResolverProvider>
        <FocusRegion
          name="Global"
          mode="column"
        >
          <div>
            <OverlayRenderer>
              <StyleExamples />

              <IconExamples />

              <ButtonExamples />

              <SelectExamples />

              <InputExamples />

              <TextareaExamples />

              <ConfirmationDialogExamples />
            </OverlayRenderer>
          </div>
        </FocusRegion>
      </HotkeysResolverProvider>
    </StylishProvider>
  )
}
