import React, { PureComponent, Fragment } from 'react'
import { inject, AppStore } from '../store'
import { readFile, sha256 } from '../../utils/browser'
import { Icon } from '../components'

export interface ISelectedFiles {
  [hash: string]: { file: File, data: Uint8Array }
}

interface IProps {
  onSelected(attachments: ISelectedFiles): void
  showLocker(show: boolean): void
}

class AttachFileButton extends PureComponent<IProps> {
  formRef = React.createRef<HTMLFormElement>()
  inputRef = React.createRef<HTMLInputElement>()

  onClick = () => this.inputRef.current!.click()

  onFilesSelected = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const {
      showLocker,
      onSelected,
    } = this.props

    const filesArr = Array.from(e.target.files!) // FileList -> Array

    if (filesArr.length) {
      showLocker(true)

      const files: ISelectedFiles = {}
      await Promise.all(filesArr.map(async (file) => {
        const data = await readFile(file)
        const hash = await sha256(data)
        files[hash] = {
          file,
          data,
        }
      }))
      onSelected(files)

      showLocker(false)
    }

    this.formRef.current!.reset()
  }

  render() {
    return (
      <Fragment>
        <Icon title="Attach files" type="paperclip" onClick={this.onClick} />

        <form hidden ref={this.formRef}>
          <input
            type="file"
            multiple
            onChange={this.onFilesSelected}
            ref={this.inputRef}
          />
        </form>
      </Fragment>
    )
  }
}

const mapStoreToProps = (store: AppStore) => ({
  showLocker: store.showLocker,
})

export default inject(mapStoreToProps, AttachFileButton)
