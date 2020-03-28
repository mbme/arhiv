import * as React from 'react'
import {
  theme,
} from './style'
import { IStyleObject } from '@v/web-utils'

const $textarea = {
  backgroundColor: theme.color.bg0,
  display: 'block',
  width: '100%',
  padding: theme.spacing.medium,

  resize: 'none',
  minHeight: '19rem',
  overflowY: 'hidden',

  border: theme.border,
  boxShadow: theme.boxShadow,
}

interface IProps {
  name: string
  value: string
  onChange(value: string): void
  placeholder?: string
  $style?: IStyleObject
}

export class Textarea extends React.PureComponent<IProps> {
  private _ref = React.createRef<HTMLTextAreaElement>()

  private _selectionStart = 0

  private _selectionEnd = 0

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

  updateHeight = () => {
    this._ref.current!.style.height = 'auto'
    this._ref.current!.style.height = `${this._ref.current!.scrollHeight}px`
  }

  onBlur = () => {
    this._selectionStart = this._ref.current!.selectionStart
    this._selectionEnd = this._ref.current!.selectionEnd
  }

  insert(str: string) {
    const { value, onChange } = this.props

    this._ref.current!.value = `${value.substring(0, this._selectionStart)}${str}${value.substring(this._selectionEnd)}`

    this._selectionStart += str.length
    this._selectionEnd = this._selectionStart

    this._ref.current!.setSelectionRange(this._selectionStart, this._selectionEnd)

    onChange(this._ref.current!.value)
  }

  focus() {
    this._ref.current!.focus()
  }

  render() {
    const {
      name,
      value,
      onChange,
      placeholder,
      $style,
    } = this.props

    return (
      <textarea
        className={$textarea.and($style).className}
        ref={this._ref}
        name={name}
        value={value}
        placeholder={placeholder}
        onChange={e => onChange(e.target.value)}
        onBlur={this.onBlur}
      />
    )
  }
}
