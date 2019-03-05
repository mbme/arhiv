import React, { PureComponent } from 'react'
import { MarkupRenderer } from '../components'
import './Note.css'

interface IProps {
  name: string
  data: string
  localLinks: { [hash: string]: string }
}

export default class Note extends PureComponent<IProps> {
  static defaultProps = {
    localLinks: {},
  }

  render() {
    const {
      name,
      data,
      localLinks,
    } = this.props

    return (
      <div>
        <h1 className="Note-title">{name}</h1>
        <MarkupRenderer value={data} localLinks={localLinks} />
      </div>
    )
  }
}
