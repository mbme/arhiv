import * as React from 'react'
import { noop, Omit } from '~/utils'
import { styleRules, style } from '~/styler'
import theme from './theme'
import { Icon } from './Icon'
import { Box } from './Box'

const $input = styleRules(
  {
    display: 'block',
    width: '100%',
    height: '100%',
    border: '0 none',
    backgroundColor: 'inherit',

    padding: theme.spacing.small,
  },
  props => (
    props.light
      ? {
        paddingTop: theme.spacing.fine,
        paddingBottom: theme.spacing.fine,
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

const $clearIcon = style({
  position: 'absolute',
  right: theme.spacing.fine,
  top: '50%',
  transform: 'translateY(-50%)',
  color: theme.color.secondary,
})

interface IProps extends Omit<React.HTMLProps<HTMLInputElement>, 'onChange'> {
  onChange(value: string): void
  autoFocus?: boolean
  light?: boolean
  onClear?(): void
  className?: string
}

export class Input extends React.PureComponent<IProps> {
  ref = React.createRef<HTMLInputElement>()

  componentDidMount() {
    if (this.props.autoFocus) this.focus()
  }

  onChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    this.props.onChange(e.target.value)
  }

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
      light,
      className,
      onClear,
      onChange,
      ...otherProps
    } = this.props

    return (
      <Box position="relative">
        <input
          ref={this.ref}
          onChange={this.onChange}
          onKeyDown={this.onKeyDown}
          className={$input(this.props, className)}
          {...otherProps}
        />

        {onClear && (
          <Icon
            type="x"
            className={$clearIcon}
            onClick={this.onClickClear}
          />
        )}
      </Box>
    )
  }
}

export const examples = {
  'Light input': (
    <Input name="input1" value="Input example (light)" light onChange={noop} />
  ),

  'Light input wiht clear': (
    <Input name="input11" value="Input example (light) with clear" light onChange={noop} onClear={noop} />
  ),

  'Input': (
    <Input name="input2" value="Input example" onChange={noop} />
  ),

  'Input with clear': (
    <Input name="input21" value="Input example with clear" onChange={noop} onClear={noop} />
  ),
}
