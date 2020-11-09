import * as React from 'react'
import { Obj, Procedure } from '@v/utils'
import {
  Input,
  Textarea,
  useForm,
  Box,
} from '@v/web-platform'
import { Frame, Action } from '../../parts'
import { DeleteDocumentButton } from './DeleteDocumentButton'
import { DocumentDataDescription } from '../../data-description'
import { DocumentData } from '../DocumentData'

interface IProps<P extends Obj> {
  data: P,
  dataDescription: DocumentDataDescription<P>,
  onSave(values: P): void
  onCancel: Procedure
  onDelete?: Procedure
}

export function CardEditor<P extends Obj>(props: IProps<P>) {
  const {
    data,
    dataDescription,
    onSave,
    onCancel,
    onDelete,
  } = props

  const {
    Form,
    values,
  } = useForm(data)

  const [preview, showPreview] = React.useState(false)

  const actions = preview ? (
    <Action
      type="action"
      onClick={() => showPreview(false)}
    >
      Back
    </Action>
  ) : (
    <>
      <Action
        type="action"
        onClick={() => onSave(values as P)}
      >
        Save Document
      </Action>

      <Action
        type="action"
        onClick={onCancel}
      >
        Cancel
      </Action>

      <Action
        type="action"
        onClick={() => showPreview(true)}
      >
        Show Preview
      </Action>

      {onDelete && <DeleteDocumentButton onConfirmed={onDelete} />}
    </>
  )

  const fields = Object.entries(dataDescription).map(([name, fieldType]) => {
    let field
    switch (fieldType.type) {
      case 'string': {
        field = (
          <Input
            label={name}
            name={name}
            placeholder={name}
          />
        )
        break
      }

      case 'markup-string': {
        field = (
          <Textarea
            label={name}
            name={name}
            placeholder={name}
          />
        )
        break
      }

      default: {
        throw new Error(`Unexpected field type: ${fieldType.type}`)
      }
    }

    return (
      <Box key={name} mb="medium">
        {field}
      </Box>
    )
  })

  return (
    <Frame
      actions={actions}
      title="Card Editor"
    >
      <Box hidden={preview}>
        <Form>
          {fields}
        </Form>
      </Box>

      {preview && (
        <DocumentData
          data={values}
          dataDescription={dataDescription}
        />
      )}
    </Frame>
  )
}
