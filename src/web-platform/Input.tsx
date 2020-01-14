import * as React from 'react'
import {
  stylish,
  $Style,
  theme,
} from './style'
import { Icon } from './Icon'
import { Box } from './Box'

const $input = stylish(
  {
    display: 'block',
    width: '100%',
    height: '100%',
    border: '0 none',
    backgroundColor: 'inherit',

    px: theme.spacing.medium,
    py: theme.spacing.small,
  },

  props => (
    props.light
      ? {
        py: theme.spacing.fine,
        backgroundColor: 'inherit',
        borderBottom: theme.border,
      }
      : {
        backgroundColor: theme.color.bg,
        boxShadow: theme.boxShadow,
        border: theme.border,
      }
  ),

  props => (
    props.onClear && {
      paddingRight: theme.spacing.medium,
    }
  ),
)

const $clearIcon = stylish({
  position: 'absolute',
  right: theme.spacing.fine,
  top: '50%',
  transform: 'translateY(-50%)',
  color: theme.color.secondary,
})

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
  $style?: $Style
  onChange(value: string): void
  autoFocus?: boolean
  light?: boolean
  onClear?(): void
}

export class Input extends React.PureComponent<IProps> {
  ref = React.createRef<HTMLInputElement>()

  componentDidMount() {
    if (this.props.autoFocus) {
      this.focus()
    }
  }

  onChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    this.props.onChange(e.target.value)
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
    const length = this.ref.current.value.length
    this.ref.current.setSelectionRange(length, length) // put cursor at the end of the input
  }

  blur = () => {
    if (!this.ref.current) {
      return
    }

    this.ref.current.blur()
  }

  onClickClear = () => {
    this.props.onChange('')
    this.props.onClear!()
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
      $style,
    } = this.props

    return (
      <Box relative>
        <input
          className={$input.and($style).with(this.props).className}
          ref={this.ref}
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

        {onClear && (
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
