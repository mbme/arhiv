import * as React from 'react'
import {
  useForm,
  Box,
} from '@v/web-platform'
import { IDocumentData } from '../../../api'
import { useDataDescription } from '../../../data-manager'
import { CardEditorFormField } from './CardEditorFormField'

interface IProps {
  documentType: string
  data: IDocumentData,
}

export const CardEditorForm = React.forwardRef(
  function CardEditorForm({ documentType, data }: IProps, ref: React.Ref<IDocumentData>) {
    const {
      Form,
      values,
    } = useForm(data)

    const {
      dataDescription,
    } = useDataDescription(documentType)

    React.useImperativeHandle(ref, () => ({ ...data, ...values }), [values])

    const fields = dataDescription.fields.map(({ name, fieldType }) => {
      return (
        <Box
          key={name}
          mb="medium"
        >
          <CardEditorFormField
            name={name}
            fieldType={fieldType}
          />
        </Box>
      )
    })

    return (
      <Form>
        {fields}
      </Form>
    )
  },
)
