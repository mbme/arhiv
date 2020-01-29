import * as React from 'react'
import { noop } from '~/utils'
import { Box, ProgressLocker } from '~/web-platform'
import { NotFound } from '~/web-app/parts'
import { useObservable } from '~/web-utils'
import { ArhivContext } from '~/web-app/arhiv-context'
import { renderCard } from '../document-types'

interface IProps {
  id: string
  focused: boolean
}

export function CardContainer({ id, focused }: IProps) {
  const ref = React.useRef<HTMLDivElement | undefined>(undefined)
  const arhiv = ArhivContext.use()
  const [document, error] = useObservable(() => arhiv.documents.getDocument$(id), [id])

  React.useEffect(() => {
    if (!focused) {
      return noop
    }

    // we use timeout here cause cards render images
    // which change scroll height and results into card not being visible
    // FIXME use context ImageMonitorProvider and make Images notify it when they're loaded
    const timeoutId = setTimeout(() => {
      ref.current?.scrollIntoView(true)
    }, 100)

    return () => clearTimeout(timeoutId)
  }, [focused])

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
      flex="0 0 auto"
      mb="large"
      width="35rem"
      maxWidth="100%"
      innerRef={ref}
    >
      {renderCard(document)}
    </Box>
  )
}
