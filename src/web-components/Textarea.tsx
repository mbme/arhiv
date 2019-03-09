import * as React from 'react'
import {
  style,
  classes,
} from 'typestyle'
import { noop } from '~/utils'
import theme from './theme'

const baseStyle = style({
  backgroundColor: theme.color.bg,
  display: 'block',
  width: '100%',
  padding: theme.spacing.medium,

  resize: 'none',
  minHeight: '19rem',
  overflowY: 'hidden',

  border: theme.border,
  boxShadow: theme.boxShadow,
})

interface IProps {
  name: string
  value: string
  onChange(value: string): void
  className?: string
}

export class Textarea extends React.PureComponent<IProps> {
  ref = React.createRef<HTMLTextAreaElement>()
  selectionStart = 0
  selectionEnd = 0

  updateHeight = () => {
    this.ref.current!.style.height = 'auto'
    this.ref.current!.style.height = `${this.ref.current!.scrollHeight}px`
  }

  componentDidMount() {
    this.updateHeight()
    window.addEventListener('resize', this.updateHeight)
  }

  componentDidUpdate() {
    this.updateHeight()
  }

  componentWillUnmount() {
    window.removeEventListener('resize', this.updateHeight)
  }

  onBlur = () => {
    this.selectionStart = this.ref.current!.selectionStart
    this.selectionEnd = this.ref.current!.selectionEnd
  }

  insert(str: string) {
    const { value, onChange } = this.props

    this.ref.current!.value = `${value.substring(0, this.selectionStart)}${str}${value.substring(this.selectionEnd)}`

    this.selectionStart += str.length
    this.selectionEnd = this.selectionStart

    this.ref.current!.setSelectionRange(this.selectionStart, this.selectionEnd)

    onChange(this.ref.current!.value)
  }

  focus() {
    this.ref.current!.focus()
  }

  render() {
    const {
      name,
      value,
      onChange,
      className,
    } = this.props

    return (
      <textarea
        className={classes(baseStyle, className)}
        ref={this.ref}
        name={name}
        value={value}
        onChange={e => onChange(e.target.value)}
        onBlur={this.onBlur}
      />
    )
  }
}

export const examples = {
  '': (
    <Textarea name="textarea" value="Textarea example" onChange={noop} />
  ),
}
