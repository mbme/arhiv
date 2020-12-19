import * as React from 'react'
import { Box } from '../Box'
import {
  StylishElement,
} from '../core'
import { Label } from '../Label'
import { mergeRefs } from '../utils'
import { useFormControl } from './Form'

type NativeProps = 'name' | 'defaultValue'

interface IProps extends Pick<React.HTMLProps<HTMLSelectElement>, NativeProps> {
  name: string
  label: string
  options: { [key: string]: string }
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

        <StylishElement
          as="select"
          ref={mergeRefs(ref, externalRef)}
          name={name}
          value={value}
          onChange={(e: React.ChangeEvent<HTMLSelectElement>) => setValue(e.target.value)}
        >
          {items}
        </StylishElement>
      </Box>
    )
  },
)
