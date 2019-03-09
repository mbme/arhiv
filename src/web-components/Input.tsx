import * as React from 'react'
import {
  style,
  classes,
} from 'typestyle'
import { noop, Omit } from '~/utils'
import theme from './theme'
import { Icon } from './Icon'

const containerStyles = (isLight?: boolean) => style(
  {
    position: 'relative',
  },
  isLight && {
    backgroundColor: 'inherit',
    borderBottom: theme.border,
  },
  !isLight && {
    backgroundColor: theme.color.bg,
    boxShadow: theme.boxShadow,
    border: theme.border,
  },
)

const inputStyles = (isLight?: boolean, withClear?: boolean) => style(
  {
    display: 'block',
    width: '100%',
    height: '100%',
    border: '0 none',
    backgroundColor: 'inherit',

    padding: theme.spacing.small,
  },
  isLight && {
    paddingTop: theme.spacing.fine,
    paddingBottom: theme.spacing.fine,
  },
  withClear && {
    paddingRight: theme.spacing.medium,
  },
)

const clearIconStyles = style({
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
      <div className={classes(containerStyles(light), className)}>
        <input
          ref={this.ref}
          onChange={this.onChange}
          onKeyDown={this.onKeyDown}
          className={classes(inputStyles(light, !!onClear))}
          {...otherProps}
        />

        {onClear && (
          <Icon
            type="x"
            className={clearIconStyles}
            onClick={this.onClickClear}
          />
        )}
      </div>
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
