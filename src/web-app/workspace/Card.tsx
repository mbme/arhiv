import * as React from 'react'
import { useArhiv } from '~/arhiv/replica'
import {
  Box,
  ProgressLocker,
  useObservable,
} from '~/web-platform'
import { prettyPrintJSON } from '~/utils'
import {
  NotFound,
} from '../parts'

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

  return (
    <Box
      as="article"
      height="100%"
      width="360px"
      overflow="auto"
      fontFamily="monospace"
      wordBreak="break-word"
      background="yellow"
    >
      {prettyPrintJSON(document.props)}
    </Box>
  )
}
