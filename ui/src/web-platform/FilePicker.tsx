import * as React from 'react'

interface IProps {
  onSelected(files: File[]): void
}

export class FilePicker extends React.PureComponent<IProps> {
  private _formRef = React.createRef<HTMLFormElement>()

  private _inputRef = React.createRef<HTMLInputElement>()

  open = () => this._inputRef.current!.click()

  private _onFilesSelected = (e: React.ChangeEvent<HTMLInputElement>) => {
    const {
      onSelected,
    } = this.props

    const filesArr = Array.from(e.target.files!) // FileList -> Array

    if (filesArr.length) {
      onSelected(filesArr)
    }

    this._formRef.current!.reset()
  }

  render() {
    return (
      <form hidden ref={this._formRef}>
        <input
          type="file"
          multiple
          onChange={this._onFilesSelected}
          ref={this._inputRef}
        />
      </form>
    )
  }
}
