import React, { PureComponent } from 'react'
import Input from './Input'
import Icon from './Icon'

interface IProps {
  placeholder: string
  filter: string
  onChange(value: string): void
}

interface IState {
  expanded: boolean
}

export default class Filter extends PureComponent<IProps, IState> {
  state = {
    expanded: false,
  }

  updateTimoutId?: number

  expand = () => this.setState({ expanded: true })

  collapse = () => this.setState({ expanded: false })

  onBlur = () => {
    if (!this.props.filter.trim()) this.collapse()
  }

  onChange = (filter: string) => {
    if (filter.trim() === this.props.filter) return

    window.clearTimeout(this.updateTimoutId)
    this.updateTimoutId = window.setTimeout(
      this.props.onChange,
      60,
      filter.trim().length ? filter : undefined
    )
  }

  componentWillUnmount() {
    window.clearTimeout(this.updateTimoutId)
  }

  render() {
    if (this.state.expanded) {
      return (
        <Input
          name="filter"
          light
          defaultValue={this.props.filter}
          placeholder={this.props.placeholder}
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
