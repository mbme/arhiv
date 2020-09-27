import * as React from 'react'
import {
  StyleArg,
  StylishElement,
} from '../core'
import { useFocusable } from '../Focus'
import { mergeRefs } from '../utils'
import { useFormControl } from './Form'

const $textarea: StyleArg = {
  backgroundColor: 'var(--color-bg0)',
  display: 'block',
  width: '100%',
  p: 'medium',

  resize: 'none',
  minHeight: '19rem',
  overflowY: 'hidden',

  border: 'default',
  boxShadow: 'default',
}

const $selected: StyleArg = {
  border: '1px solid red',
}

interface IProps {
  name: string
  placeholder?: string
  $styles?: StyleArg[]
}

export const Textarea = React.forwardRef<HTMLTextAreaElement, IProps>(function Textarea(props, externalRef) {
  const {
    name,
    placeholder,
    $styles = [],
  } = props

  const ref = React.useRef<HTMLTextAreaElement>(null)
  const isSelected = useFocusable(ref)

  const {
    value,
    setValue,
  } = useFormControl(name)

  const updateHeight = () => {
    ref.current!.style.height = 'auto'
    ref.current!.style.height = `${ref.current!.scrollHeight}px`
  }

  React.useEffect(() => {
    window.addEventListener('resize', updateHeight)

    return () => {
      window.removeEventListener('resize', updateHeight)
    }
  }, [])

  React.useEffect(() => {
    updateHeight()
  }, [value])

  const onKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.key === 'Escape') {
      (e.target as HTMLTextAreaElement).blur()
    }
  }

  return (
    <StylishElement
      ref={mergeRefs(ref, externalRef)}
      as="textarea"
      $styles={[$textarea, isSelected && $selected, ...$styles]}
      name={name}
      value={value}
      placeholder={placeholder}
      onChange={(e: React.ChangeEvent<HTMLTextAreaElement>) => setValue(e.target.value)}
      onKeyDown={onKeyDown}
    />
  )
})
