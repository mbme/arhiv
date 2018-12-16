import React, { PureComponent, createRef } from 'react'
import { classNames, Omit } from '../../utils'
import Icon from './Icon'
import './Input.css'

interface IProps extends Omit<React.HTMLProps<HTMLInputElement>, 'onChange'> {
  onChange: (value: string) => void
  autoFocus?: boolean
  light?: boolean
  onClear?: () => void
  className?: string
}

export default class Input extends PureComponent<IProps, {}> {
  ref = createRef<HTMLInputElement>()

  componentDidMount() {
    if (this.props.autoFocus) this.focus()
  }

  onChange = (e: React.ChangeEvent<HTMLInputElement>) => this.props.onChange(e.target.value)

  onKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Escape') this.blur()
  }

  focus = () => {
    if (!this.ref.current) return

    this.ref.current.focus()
    const length = this.ref.current.value.length
    this.ref.current.setSelectionRange(length, length) // put cursor at the end of the input
  }

  blur = () => {
    if (!this.ref.current) return

    this.ref.current.blur()
  }

  onClickClear = () => {
    this.props.onChange('')
    this.props.onClear!()
  }

  render() {
    const {
      onChange,
      light,
      className,
      onClear,
      ...other
    } = this.props

    return (
      <div className={classNames('Input-container', { 'is-light': light }, className)}>
        <input
          ref={this.ref}
          onChange={this.onChange}
          onKeyDown={this.onKeyDown}
          className={classNames('Input-input', { 'is-light': light, 'is-with-clear': onClear })}
          {...other}
        />
        {onClear && <Icon type="x" className="Input-clear" onClick={this.onClickClear} />}
      </div>
    )
  }
}
