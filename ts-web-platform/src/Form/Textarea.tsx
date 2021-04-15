import * as React from 'react'
import { Box } from '../Box'
import {
  StyleArg,
  StylishElement,
} from '../core'
import { Label } from '../Label'
import { Spacer } from '../Layout'
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
  borderRadius: 'var(--border-radius-form)',
}

interface IProps {
  name: string
  label: string
  placeholder?: string
  $styles?: StyleArg[]
}

export const Textarea = React.forwardRef(
  function Textarea(props: IProps, externalRef: React.Ref<HTMLTextAreaElement>) {
    const {
      name,
      label,
      placeholder,
      $styles = [],
    } = props

    const containerRef = React.useRef<HTMLDivElement>(null)
    const ref = React.useRef<HTMLTextAreaElement>(null)

    const {
      value,
      setValue,
    } = useFormControl(name)

    const updateHeight = () => {
      // preserve height between updates
      containerRef.current!.style.height = `${containerRef.current!.scrollHeight}px`

      ref.current!.style.height = 'auto'
      ref.current!.style.height = `${ref.current!.scrollHeight}px`

      containerRef.current!.style.height = 'auto'
    }

    React.useEffect(() => {
      window.addEventListener('resize', updateHeight, { passive: true })

      return () => {
        window.removeEventListener('resize', updateHeight)
      }
    }, [])

    React.useEffect(() => {
      updateHeight()
    }, [value])

    const onChange = (e: React.ChangeEvent<HTMLTextAreaElement>) => setValue(e.target.value)

    return (
      <Box ref={containerRef}>
        <Label>{label}</Label>
        <Spacer height="small" />

        <StylishElement
          ref={mergeRefs(ref, externalRef)}
          as="textarea"
          $styles={[$textarea, ...$styles]}
          name={name}
          value={value}
          placeholder={placeholder}
          onChange={onChange}
        />
      </Box>
    )
  },
)
