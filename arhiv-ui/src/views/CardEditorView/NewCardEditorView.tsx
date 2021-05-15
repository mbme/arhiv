import * as React from 'react'
import {
  RouterContext,
} from '@v/web-utils'
import { Dict } from '@v/utils'
import { API } from '@v/arhiv-api'
import { CardLoader, FrameTitle } from '../../parts'
import { useDataDescription } from '../../data-manager'
import { CardEditor } from './CardEditor'

interface IProps {
  documentType: string
}

export function NewCardEditorView({ documentType }: IProps) {
  const router = RouterContext.use()
  const { mandatoryFields } = useDataDescription(documentType)

  const args: Dict<any> = {}

  for (const mandatoryField of mandatoryFields) {
    const param = router.location$.value.params.find(item => item.name === mandatoryField)

    if (!param) {
      throw new Error(`Manadatory query param ${mandatoryField} is missing`)
    }

    args[mandatoryField] = param.value
  }

  return (
    <CardLoader
      createDocument={() => API.create({ documentType, args })}
    >
      {document => (
        <>
          <FrameTitle>
            New {document.documentType}
          </FrameTitle>

          <CardEditor
            document={document}
            onSave={() => router.replace({ path: `/documents/${document.id}` })}
            onCancel={() => router.goBack()}
          />
        </>
      )}
    </CardLoader>
  )
}
