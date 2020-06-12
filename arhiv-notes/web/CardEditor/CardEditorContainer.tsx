import * as React from 'react'
import { createLogger } from '@v/logger'
import {
  ProgressLocker,
} from '@v/web-platform'
import {
  usePromise,
  RouterContext,
} from '@v/web-utils'
import { API } from '../notes'
import { CardEditor } from './CardEditor'
import { selectLinks, parseMarkup } from '../markup-parser'

const log = createLogger('CardEditor')

async function extractRefs(data: string) {
  const attachmentRefs = new Set<string>()
  const documentRefs = new Set<string>()

  const markup = parseMarkup(data)

  for (const link of selectLinks(markup)) {
    const id = link.link

    if (await API.get_note(id)) {
      documentRefs.add(id)
    } else if (await API.get_attachment(id)) {
      attachmentRefs.add(id)
    } else {
      log.warn(`document references unknown entity ${id}`)
    }
  }

  return {
    refs: Array.from(documentRefs),
    attachmentRefs: Array.from(attachmentRefs),
  }
}

interface IProps {
  id?: string
}

export function CardEditorContainer({ id }: IProps) {
  const router = RouterContext.use()

  const [note] = usePromise(() => {
    if (id) {
      return API.get_note(id)
    }

    return API.create_note()
  }, [id])

  if (!note) { // FIXME not found
    return (
      <ProgressLocker />
    )
  }

  const onSave = async (name: string, data: string) => {
    const {
      refs,
      attachmentRefs,
    } = await extractRefs(data)

    await API.put_note({
      ...note,
      refs,
      attachmentRefs,
      data: {
        name,
        data,
      },
    })

    router.push({ path: `/${note.id}` })
  }

  const onDelete = async () => {
    await API.put_note({
      ...note,
      archived: true,
    })

    // FIXME go back in history or go home
  }

  const onCancel = () => {
    // FIXME go back in history or go home
  }

  return (
    <CardEditor
      name={note.data.name}
      data={note.data.data}
      onSave={onSave}
      onDelete={id ? onDelete : undefined}
      onCancel={onCancel}
    />
  )
}
