import * as React from 'react'
import {
  Box,
  ProgressLocker,
  useObservable,
  Heading,
} from '~/web-platform'
import { prettyPrintJSON } from '~/utils'
import { DocumentNote } from '~/arhiv/replica'
import { useArhiv } from '~/arhiv/useArhiv'
import {
  NotFound,
  Markup,
} from '../../parts'
import { CardFrame } from './CardFrame'

interface IProps {
  id: string
}

// Card
export function Card({ id }: IProps) {
  const arhiv = useArhiv()
  const [document, error] = useObservable(() => arhiv.documents.getDocument$(id), [id])

  if (error) {
    return NotFound
  }

  if (!document) {
    return (
      <ProgressLocker />
    )
  }

  if (document instanceof DocumentNote) {
    return (
      <CardFrame document={document}>
        <Heading
          letterSpacing="1.4px"
          fontSize="large"
        >
          {document.name}
        </Heading>

        <Markup value={document.data} />
      </CardFrame>
    )
  }

  return (
    <CardFrame document={document}>
      <Box
        fontFamily="mono"
        wordBreak="break-word"
      >
        {prettyPrintJSON(document.props)}
      </Box>
    </CardFrame>
  )
}
