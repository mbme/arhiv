import * as React from 'react'

import { PlatformProvider } from './PlatformProvider'

import { ButtonExamples } from './Button.examples'
import { SelectExamples } from './Form/Select.examples'
import { InputExamples } from './Form/Input.examples'
import { TextareaExamples } from './Form/Textarea.examples'
import { IconExamples } from './Icon.examples'
import { LinkExamples } from './router/Link.examples'
import { StyleExamples } from './core/examples'
import { ConfirmationDialogExamples } from './Modal/ConfirmationDialog.examples'
import { HeadingExamples } from './Heading.examples'
import { CodeBlockExamples } from './CodeBlock.examples'

export function Library() {
  return (
    <PlatformProvider>
      <StyleExamples />

      <HeadingExamples />

      <IconExamples />

      <ButtonExamples />

      <LinkExamples />

      <CodeBlockExamples />

      <SelectExamples />

      <InputExamples />

      <TextareaExamples />

      <ConfirmationDialogExamples />
    </PlatformProvider>
  )
}
