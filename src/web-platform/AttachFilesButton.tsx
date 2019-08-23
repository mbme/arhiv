import * as React from 'react'
import { Icon } from './Icon'

interface IProps {
  onSelected(files: File[]): void
}

export class AttachFilesButton extends React.PureComponent<IProps> {
  formRef = React.createRef<HTMLFormElement>()
  inputRef = React.createRef<HTMLInputElement>()

  onClick = () => this.inputRef.current!.click()

  onFilesSelected = (e: React.ChangeEvent<HTMLInputElement>) => {
    const {
      onSelected,
    } = this.props

    const filesArr = Array.from(e.target.files!) // FileList -> Array

    if (filesArr.length) {
      onSelected(filesArr)
    }

    this.formRef.current!.reset()
  }

  render() {
    return (
      <>
        <Icon title="Attach files" type="paperclip" onClick={this.onClick} />

        <form hidden ref={this.formRef}>
          <input
            type="file"
            multiple
            onChange={this.onFilesSelected}
            ref={this.inputRef}
          />
        </form>
      </>
    )
  }
}
