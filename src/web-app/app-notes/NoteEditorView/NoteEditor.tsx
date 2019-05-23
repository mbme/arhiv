import * as React from 'react'
import { style } from 'typestyle'
import {
  useRouter,
} from '~/web-router'
import {
  Icon,
  Button,
  Input,
  Textarea,
  section,
  theme,
} from '~/web-components'
import { Note } from '~/isodb-web-client'
import {
  Toolbar,
} from '../../parts'
import { Note as NoteRenderer } from '../Note'

interface IProps {
  note: Note,
}

const titleStyle = style({
  textAlign: 'center',
  letterSpacing: '1.4px',
  fontWeight: 'bold',
  fontSize: theme.fontSize.large,

  marginBottom: theme.spacing.medium,
})

export function NoteEditor({ note }: IProps) {
  const router = useRouter()

  const [isPreview, setPreview] = React.useState(false)
  const [name, setName] = React.useState(note.name)
  const [data, setData] = React.useState(note.data)

  const onPreview = () => setPreview(!isPreview)
  const left = (
    <Icon
      title="Preview"
      type={isPreview ? 'eye-off' : 'eye'}
      onClick={onPreview}
    />
  )

  const isValid = name && name !== note.name || data !== note.data
  const onCancel = () => router.push(note.isNew() ? { path: '/notes' } : { path: '/note', params: { id: note.id } })
  const onSave = () => {
    note.name = name
    note.data = data
    note.save()
    router.push({ path: '/note', params: { id: note.id } })
  }

  const right = (
    <>
      <Button onClick={onCancel}>
        Cancel
      </Button>

      <Button primary onClick={onSave} disabled={!isValid}>
        Save
      </Button>
    </>
  )

  return (
    <>
      <Toolbar left={left} right={right} />

      <div className={section} hidden={isPreview}>
        <Input
          className={titleStyle}
          name="name"
          value={name}
          onChange={setName}
          autoFocus
        />
      </div>

      <div className={section} hidden={isPreview}>
        <Textarea
          name="data"
          value={data}
          onChange={setData}
        />
      </div>

      {isPreview && <NoteRenderer name={name} data={data} />}
    </>
  )
}
