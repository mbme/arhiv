import React, { PureComponent, createRef } from 'react'
import './Textarea.css'

interface IProps {
  name: string
  value: string
  onChange(value: string): void
}

export default class Textarea extends PureComponent<IProps> {
  ref = createRef<HTMLTextAreaElement>()
  selectionStart = 0
  selectionEnd = 0

  updateHeight = () => {
    this.ref.current!.style.height = 'auto'
    this.ref.current!.style.height = `${this.ref.current!.scrollHeight}px'`
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
    const { name, value, onChange } = this.props

    return (
      <textarea
        className="Textarea"
        ref={this.ref}
        name={name}
        value={value}
        onChange={e => onChange(e.target.value)}
        onBlur={this.onBlur}
      />
    )
  }
}
