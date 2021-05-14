import * as React from 'react'
import { Dict } from '@v/utils'
import { Box } from '../Box'
import {
  StyleArg,
  StylishElement,
} from '../core'
import { Label } from '../Label'
import { Spacer } from '../Layout'
import { mergeRefs } from '../utils'
import { useFormControl } from './Form'

const $style: StyleArg = {
  backgroundColor: 'var(--color-bg0)',
  boxShadow: 'default',
  border: 'default',
  borderRadius: 'var(--border-radius-form)',

  px: 'medium',
  py: 'small',

  minWidth: '16rem',
}

type NativeProps = 'name' | 'defaultValue'

interface IProps extends Pick<React.HTMLProps<HTMLSelectElement>, NativeProps> {
  name: string
  label: string
  options: Dict<string | undefined>
}

export const Select = React.forwardRef(
  function Select({ options, name, label }: IProps, externalRef: React.Ref<HTMLSelectElement>) {
    const {
      value,
      setValue,
    } = useFormControl(name)

    const ref = React.useRef<HTMLSelectElement>(null)

    const items = Object.entries(options).map(([key, label]) => (
      <option key={key} value={key}>
        {label}
      </option>
    ))

    return (
      <Box>
        <Label>{label}</Label>
        <Spacer height="small" />

        <StylishElement
          as="select"
          ref={mergeRefs(ref, externalRef)}
          name={name}
          value={value}
          onChange={(e: React.ChangeEvent<HTMLSelectElement>) => setValue(e.target.value)}
          $style={$style}
        >
          {items}
        </StylishElement>
      </Box>
    )
  },
)
