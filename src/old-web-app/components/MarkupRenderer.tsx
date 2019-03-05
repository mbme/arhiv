import React, { PureComponent } from 'react'
import { parse, Node } from '../../v-parser'
import './MarkupRenderer.css'

interface IProps {
  value: string
  localLinks: { [hash: string]: string }
}

export default class MarkupRenderer extends PureComponent<IProps> {
  getFileUrl = (fileId: string) => {
    const localFileUrl = this.props.localLinks[fileId]

    return localFileUrl || `/api/file?fileId=${fileId}`
  }

  renderItem = (item: Node): React.ReactNode => {
    if (typeof item === 'string') {
      return item
    }

    switch (item.type) {
      case 'Document': {
        return React.createElement('article', { className: 'Note-document' }, ...item.items.map(this.renderItem))
      }

      case 'Paragraph': {
        return React.createElement('p', {}, ...item.items.map(this.renderItem))
      }

      case 'Header':
        return (
          <h1>{item.text}</h1>
        )

      case 'Mono':
        return (
          <code>{item.text}</code>
        )

      case 'Bold':
        return (
          <strong>{item.text}</strong>
        )

      case 'Link': {
        const url = item.isInternal ? this.getFileUrl(item.address) : item.address

        if (item.linkType === 'image') {
          return (
            <img className="Note-image" alt={item.name} src={url} />
          )
        }

        return (
          <a href={url}>{item.name}</a>
        )
      }

      default:
        return item
    }
  }

  render() {
    return this.renderItem(parse(this.props.value))
  }
}
