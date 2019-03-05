import React, { PureComponent, Fragment } from 'react'
import {
  createLink,
  createImageLink,
  extractFileIds,
  parse,
} from '../../v-parser'
import {
  Button,
  Textarea,
  Input,
  Icon,
} from '../components'
import { Toolbar } from '../parts'
import Note from './Note'
import AttachFileButton, { ISelectedFiles } from './AttachFileButton'
import DeleteNoteButton from './DeleteNoteButton'
import './Note.css'

const isImage = (name: string) => ['.png', '.jpg', '.jpeg'].reduce((acc, ext) => acc || name.endsWith(ext), false)

interface IProps {
  id?: string,
  name: string
  data: string
  onSave(name: string, data: string, assets: File[]): void
  onCancel(): void
}

interface IState {
  preview: boolean
  name: string
  data: string
  localFiles: ISelectedFiles
  localLinks: { [hash: string]: string }
}

export default class NoteEditor extends PureComponent<IProps, IState> {
  state: IState = {
    preview: false,
    name: this.props.name,
    data: this.props.data,
    localFiles: {},
    localLinks: {},
  }

  textAreaRef = React.createRef<Textarea>()

  hasChanges = () => this.state.name !== this.props.name || this.state.data !== this.props.data
  changeName = (name: string) => this.setState({ name })
  changeData = (data: string) => this.setState({ data })

  togglePreview = () => this.setState(state => ({ preview: !state.preview }))

  onFilesSelected = async (files: ISelectedFiles) => {
    const links = []

    const newLocalFiles = {
      ...this.state.localFiles,
    }
    const newLocalLinks = {
      ...this.state.localLinks,
    }

    for (const [hash, { file, data }] of Object.entries(files)) {
      links.push(
        isImage(file.name)
          ? createImageLink(file.name, hash)
          : createLink(file.name, hash)
      )

      newLocalFiles[hash] = newLocalFiles[hash] || { file, data }
      newLocalLinks[hash] = newLocalLinks[hash] || URL.createObjectURL(file)
    }

    this.setState({
      localFiles: newLocalFiles,
      localLinks: newLocalLinks,
    })

    this.textAreaRef.current!.insert(links.join(' '))
    this.textAreaRef.current!.focus()
  }

  onSave = async () => {
    const {
      name,
      data,
      localFiles,
    } = this.state

    const ids = extractFileIds(parse(data))

    // TODO filter out known files
    const assets = Object.entries(localFiles).filter(([id]) => ids.includes(id)).map(([, file]) => file.file)

    this.props.onSave(name, data, assets)
  }

  componentWillUnmount() {
    for (const url of Object.values(this.state.localLinks)) {
      URL.revokeObjectURL(url)
    }
  }

  render() {
    const {
      preview,
      name,
      data,
      localLinks,
    } = this.state

    const {
      id,
      onCancel,
    } = this.props

    const isValid = name && name !== this.props.name || data !== this.props.data

    const leftIcons = (
      <Fragment>
        {id && <DeleteNoteButton id={id} />}

        <AttachFileButton onSelected={this.onFilesSelected} />

        <Icon title="Preview" type={preview ? 'eye-off' : 'eye'} onClick={this.togglePreview} />
      </Fragment>
    )

    const rightIcons = (
      <Fragment>
        <Button onClick={onCancel}>Cancel</Button>

        <Button primary onClick={this.onSave} disabled={!isValid}>Save</Button>
      </Fragment>
    )

    return (
      <Fragment>
        <Toolbar left={leftIcons} right={rightIcons} />

        <div className="g-section" hidden={preview}>
          <Input className="Note-title" name="name" value={name} onChange={this.changeName} autoFocus />
        </div>

        <div className="g-section" hidden={preview}>
          <Textarea
            name="data"
            value={data}
            onChange={this.changeData}
            ref={this.textAreaRef}
          />
        </div>

        {preview && <Note name={name} data={data} localLinks={localLinks} />}
      </Fragment>
    )
  }
}
