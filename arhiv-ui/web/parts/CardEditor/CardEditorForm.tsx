import * as React from 'react'
import { Obj } from '@v/utils'
import {
  Input,
  Textarea,
  useForm,
  Box,
} from '@v/web-platform'
import { DocumentDataDescription } from '../../data-description'

interface IProps<P extends Obj> {
  data: P,
  dataDescription: DocumentDataDescription<P>,
}

export const CardEditorForm = React.forwardRef(
  function CardEditorForm<P extends Obj>({ data, dataDescription }: IProps<P>, ref: React.Ref<P>) {
    const {
      Form,
      values,
    } = useForm(data)

    React.useImperativeHandle(ref, () => values as P, [values])

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
      <Form>
        {fields}
      </Form>
    )
  },
)
