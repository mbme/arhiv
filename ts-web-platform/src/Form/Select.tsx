import * as React from 'react'
import { Box } from '../Box'
import {
  StyleArg,
  StylishElement,
} from '../core'
import { useFocusable, useFocusOnActivate } from '../Focus'
import { Label } from '../Label'
import { mergeRefs } from '../utils'
import { useFormControl } from './Form'

const $selected: StyleArg = {
  border: '1px solid red',
}

type NativeProps = 'name' | 'defaultValue'

interface IProps extends Pick<React.HTMLProps<HTMLSelectElement>, NativeProps> {
  name: string
  label: string
  options: { [key: string]: string }
}

export const Select = React.forwardRef<HTMLSelectElement, IProps>(
  function Select({ options, name, label }: IProps, externalRef) {
    const {
      value,
      setValue,
    } = useFormControl(name)

    const ref = React.useRef<HTMLSelectElement>(null)
    const isSelected = useFocusable(ref)

    useFocusOnActivate(ref)

    const items = Object.entries(options).map(([key, label]) => (
      <option key={key} value={key}>
        {label}
      </option>
    ))

    return (
      <Box>
        <Label>{label}</Label>

        <StylishElement
          as="select"
          ref={mergeRefs(ref, externalRef)}
          name={name}
          value={value}
          onChange={(e: React.ChangeEvent<HTMLSelectElement>) => setValue(e.target.value)}
          $style={isSelected ? $selected : undefined}
        >
          {items}
        </StylishElement>
      </Box>
    )
  } ,
)
