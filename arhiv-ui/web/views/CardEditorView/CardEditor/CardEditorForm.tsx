import * as React from 'react'
import {
  useForm,
  Box,
} from '@v/web-platform'
import { IDocumentData } from '../../../api'
import { useDataDescription } from '../../../data-manager'
import { CardEditorFormField } from './CardEditorFormField'

interface IProps {
  data: IDocumentData,
}

export const CardEditorForm = React.forwardRef(
  function CardEditorForm({ data }: IProps, ref: React.Ref<IDocumentData>) {
    const {
      Form,
      values,
    } = useForm(data)

    const {
      dataDescription,
    } = useDataDescription(data.type)

    React.useImperativeHandle(ref, () => ({ ...data, ...values }), [values])

    const fields = Object.entries(dataDescription.fields).map(([name, field]) => {
      return (
        <Box key={name} mb="medium">
          <CardEditorFormField
            name={name}
            fieldType={field.fieldType}
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
