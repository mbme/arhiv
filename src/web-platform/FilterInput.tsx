import * as React from 'react'
import { Input } from './Input'
import { Icon } from './Icon'

interface IProps {
  placeholder: string
  filter: string
  alwaysExpanded?: boolean
  onChange(value?: string): void
}

interface IState {
  expanded: boolean
}

export class FilterInput extends React.PureComponent<IProps, IState> {
  state = {
    expanded: this.props.alwaysExpanded || false,
  }

  updateTimoutId?: number

  expand = () => {
    this.setState({ expanded: true })
  }

  collapse = () => {
    const {
      alwaysExpanded,
    } = this.props

    this.setState({
      expanded: alwaysExpanded || false,
    })
  }

  onBlur = () => {
    const {
      filter,
    } = this.props

    if (!filter) {
      this.collapse()
    }
  }

  onChange = (newFilter: string) => {
    const {
      filter,
      onChange,
    } = this.props

    if (newFilter.trim() === filter) {
      return
    }

    window.clearTimeout(this.updateTimoutId)
    this.updateTimoutId = window.setTimeout(
      onChange,
      60,
      newFilter.trim().length ? newFilter : undefined,
    )
  }

  componentWillUnmount() {
    window.clearTimeout(this.updateTimoutId)
  }

  render() {
    const {
      filter,
      placeholder,
    } = this.props

    const {
      expanded,
    } = this.state

    if (expanded || filter) {
      return (
        <Input
          name="filter"
          light
          defaultValue={filter}
          placeholder={placeholder}
          onChange={this.onChange}
          onClear={this.collapse}
          onBlur={this.onBlur}
          autoFocus
        />
      )
    }

    return (
      <Icon type="search" onClick={this.expand} />
    )
  }
}
