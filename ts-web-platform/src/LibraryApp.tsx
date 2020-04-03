import * as React from 'react'
import * as ReactDOM from 'react-dom'

import { injectGlobalStyles } from '@v/web-utils'

import { StylishProvider } from './core'
import { OverlayRenderer } from './Overlay'
import { globalStyles } from './core/global-styles'
import { ButtonExamples } from './Button.examples'
import { SelectExamples } from './Select.examples'
import { InputExamples } from './Input.examples'
import { TextareaExamples } from './Textarea.examples'
import { IconExamples } from './Icon.examples'
import { StyleExamples } from './core/examples'
import { ConfirmationDialogExamples } from './Overlay/ConfirmationDialog.examples'

injectGlobalStyles(`
  ${globalStyles}

  #root {
    height: 100vh;
    max-width: 50rem;
    margin: 0 auto;
    visibility: hidden;
  }
`)

const rootEl = document.getElementById('root')
if (!rootEl) {
  throw new Error("Can't find #root element")
}

ReactDOM.render(
  <React.StrictMode>
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
  </React.StrictMode>,
  rootEl,
  () => {
    rootEl.style.visibility = 'visible'
  },
)
