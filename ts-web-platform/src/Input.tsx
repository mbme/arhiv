import * as React from 'react'
import { Icon } from './Icon'
import { Box } from './Box'
import {
  StyleArg,
  StylishElement,
} from './core'

const getStyles = (props: IProps): StyleArg[] => [
  {
    display: 'block',
    width: '100%',
    height: '100%',
    border: '0 none',
    backgroundColor: 'inherit',

    px: 'medium',
    py: 'small',
  },

  props.light ? {
    py: 'fine',
    backgroundColor: 'inherit',
    borderBottom: 'default',
  } : {
    backgroundColor: 'bg0',
    boxShadow: 'default',
    border: 'default',
  },

  props.onClear && {
    paddingRight: 'medium',
  },
]

const $clearIcon: StyleArg = {
  position: 'absolute',
  right: 'fine',
  top: '50%',
  transform: 'translateY(-50%)',
  color: 'secondary',
}

type NativeProps =
  'type'
  | 'name'
  | 'value'
  | 'onKeyDown'
  | 'onBlur'
  | 'defaultValue'
  | 'placeholder'
  | 'autoComplete'

interface IProps extends Pick<React.HTMLProps<HTMLInputElement>, NativeProps> {
  onChange(value: string): void
  autoFocus?: boolean
  light?: boolean
  onClear?(): void
}

export class Input extends React.PureComponent<IProps> {
  ref = React.createRef<HTMLInputElement>()

  componentDidMount() {
    const {
      autoFocus,
    } = this.props

    if (autoFocus) {
      this.focus()
    }
  }

  onChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const {
      onChange,
    } = this.props

    onChange(e.target.value)
  }

  onKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    const {
      onKeyDown,
    } = this.props

    if (e.key === 'Escape') {
      this.blur()
    }

    if (onKeyDown) {
      onKeyDown(e)
    }
  }

  focus = () => {
    if (!this.ref.current) {
      return
    }

    this.ref.current.focus()
    const { length } = this.ref.current.value
    this.ref.current.setSelectionRange(length, length) // put cursor at the end of the input
  }

  blur = () => {
    if (!this.ref.current) {
      return
    }

    this.ref.current.blur()
  }

  onClickClear = () => {
    const {
      onChange,
      onClear,
    } = this.props

    onChange('')
    onClear!()
  }

  render() {
    const {
      onClear,
      autoFocus,
      type,
      name,
      value,
      defaultValue,
      autoComplete,
      placeholder,
      onBlur,
    } = this.props

    return (
      <Box
        relative
        width="100%"
      >
        <StylishElement
          as="input"
          $styles={getStyles(this.props)}
          innerRef={this.ref}
          type={type}
          name={name}
          value={value}
          defaultValue={defaultValue}
          autoComplete={autoComplete}
          placeholder={placeholder}
          onChange={this.onChange}
          onKeyDown={this.onKeyDown}
          autoFocus={autoFocus}
          onBlur={onBlur}
        />

        {onClear && value && (
          <Icon
            type="x"
            $styles={[$clearIcon]}
            onClick={this.onClickClear}
          />
        )}
      </Box>
    )
  }
}
