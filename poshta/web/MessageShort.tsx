import * as React from 'react'
import {
  Box,
  Row,
  Spacer,
  Label,
} from '@v/web-platform'
import { GmailMessage } from './gmail'
import { ChronoFormatter } from '@v/chrono'

const dateFormat = new ChronoFormatter('HH:mm DD.MM.YYYY')

interface IProps {
  message: GmailMessage
  focused?: boolean
}

export function MessageShort({ message, focused }: IProps ) {
  const ref = React.useRef<HTMLDivElement>(null)

  React.useEffect(() => {
    if (focused && ref.current) {
      ref.current.scrollIntoView({
        behavior: 'smooth',
        block: 'nearest',
      })
    }
  }, [focused, ref.current])

  return (
    <Box
      mb="large"
      p="medium"
      border="1px solid black"
      bgColor={focused ? 'yellow' : undefined}
      ref={ref}
    >
      <Row alignX="left">
        {message.labels.map(label => (
          <Box key={label} bgColor="yellow" mr="fine">
            {label}
          </Box>
        ))}
        <Spacer />
        <Label>
          {dateFormat.format(message.date)}
        </Label>
      </Row>

      <h3>{message.subject}</h3>

      <Box
        dangerouslySetInnerHTML={{ __html: message.snippet }}
        mb="large"
      />

      {message.attachments.length > 0 && (
        <Label>{message.attachments.length} attachments</Label>
      )}
      <Box>
        <b>From: </b> {message.from}
      </Box>
      <Box>
        <b>To: </b> {message.to}
      </Box>
    </Box>
  )
}
