import * as React from 'react'

import { HotkeysResolverProvider, RouterProvider } from '@v/web-utils'
import { StylishProvider } from './core'
import { OverlayRenderer } from './Modal'
import { ButtonExamples } from './Button.examples'
import { SelectExamples } from './Form/Select.examples'
import { InputExamples } from './Form/Input.examples'
import { TextareaExamples } from './Form/Textarea.examples'
import { IconExamples } from './Icon.examples'
import { LinkExamples } from './Link.examples'
import { StyleExamples } from './core/examples'
import { ConfirmationDialogExamples } from './Modal/ConfirmationDialog.examples'
import { FocusRegion } from './Focus'

export function Library() {
  return (
    <RouterProvider hashBased>
      <StylishProvider>
        <HotkeysResolverProvider>
          <FocusRegion
            name="Global"
            mode="column"
          >
            <OverlayRenderer>
              <StyleExamples />

              <IconExamples />

              <ButtonExamples />

              <LinkExamples />

              <SelectExamples />

              <InputExamples />

              <TextareaExamples />

              <ConfirmationDialogExamples />
            </OverlayRenderer>
          </FocusRegion>
        </HotkeysResolverProvider>
      </StylishProvider>
    </RouterProvider>
  )
}
