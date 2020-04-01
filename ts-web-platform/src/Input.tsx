import * as React from 'react'
import { theme } from './style'
import { Icon } from './Icon'
import { Box } from './Box'
import { StylishElement, mergeStyles } from '@v/web-utils'

const getStyles = (props: IProps) => mergeStyles([
  {
    display: 'block',
    width: '100%',
    height: '100%',
    border: '0 none',
    backgroundColor: 'inherit',

    px: theme.spacing.medium,
    py: theme.spacing.small,
  },

  props.light ? {
    py: theme.spacing.fine,
    backgroundColor: 'inherit',
    borderBottom: theme.border,
  } : {
    backgroundColor: theme.color.bg0,
    boxShadow: theme.boxShadow,
    border: theme.border,
  },

  props.onClear && {
    paddingRight: theme.spacing.medium,
  },
])

const $clearIcon = {
  position: 'absolute',
  right: theme.spacing.fine,
  top: '50%',
  transform: 'translateY(-50%)',
  color: theme.color.secondary,
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
      <Box relative>
        <StylishElement
          as="input"
          $style={getStyles(this.props)}
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
            $style={$clearIcon}
            onClick={this.onClickClear}
          />
        )}
      </Box>
    )
  }
}
