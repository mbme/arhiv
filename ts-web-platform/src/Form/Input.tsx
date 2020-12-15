import * as React from 'react'
import { Icon } from '../Icon'
import { Box } from '../Box'
import {
  StyleArg,
  StylishElement,
} from '../core'
import { mergeRefs } from '../utils'
import { useFormControl } from './Form'
import { useFocusable, useFocusOnActivate } from '../Focus'
import { Label } from '../Label'
import { Procedure } from '@v/utils'

const getStyles = (withClear?: boolean, isSelected?: boolean): StyleArg[] => [
  {
    display: 'block',
    width: '100%',
    height: '100%',
    backgroundColor: 'var(--color-bg0)',
    boxShadow: 'default',
    border: 'default',

    px: 'medium',
    py: 'small',
  },

  withClear && {
    paddingRight: 'medium',
  },

  isSelected && {
    border: 'active',
  },
]

const $clearIcon: StyleArg = {
  position: 'absolute',
  right: 'fine',
  top: '50%',
  transform: 'translateY(-50%)',
  color: 'var(--color-secondary)',
}

type NativeProps =
  'type'
  | 'name'
  | 'defaultValue'
  | 'placeholder'
  | 'autoComplete'
  | 'readOnly'

interface IProps extends Pick<React.HTMLProps<HTMLInputElement>, NativeProps> {
  name: string
  label: string
  withClear?: boolean
  onFocus?: Procedure
}

export const Input = React.forwardRef(
  function Input(props: IProps, externalRef: React.Ref<HTMLInputElement>) {
    const {
      type,
      name,
      label,
      defaultValue,
      placeholder,
      autoComplete,
      withClear,
      onFocus,
    } = props

    const {
      value,
      setValue,
    } = useFormControl(name)

    const ref = React.useRef<HTMLInputElement>(null)
    const isSelected = useFocusable(ref, { onFocus })

    useFocusOnActivate(ref)

    return (
      <Box
        relative
        width="100%"
      >
        {label && (
          <Label>{label}</Label>
        )}

        <StylishElement
          ref={mergeRefs(ref, externalRef)}
          as="input"
          $styles={getStyles(props.withClear, isSelected)}
          type={type}
          name={name}
          value={value}
          defaultValue={defaultValue}
          autoComplete={autoComplete}
          placeholder={placeholder}
          onChange={(e: React.ChangeEvent<HTMLInputElement>) => setValue(e.target.value)}
        />

        {withClear && value && (
          <Icon
            type="x"
            $styles={[$clearIcon]}
            onClick={() => setValue('')}
          />
        )}
      </Box>
    )
  },
)
