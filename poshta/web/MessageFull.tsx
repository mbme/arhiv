import * as React from 'react'
import {
  Text,
  Box,
  Row,
  Spacer,
} from '@v/web-platform'
import { GmailMessage } from './gmail'
import { ChronoFormatter } from '@v/chrono'

const dateFormat = new ChronoFormatter('HH:mm DD.MM.YYYY')

interface IProps {
  message: GmailMessage
}

export function MessageFull({ message }: IProps ) {
  const body = React.useMemo(() => {
    if (message.hasHTMLBody()) {
      return (
        <Box
          dangerouslySetInnerHTML={{ __html: message.getHTMLBody() }}
        />
      )
    }

    if (message.hasTextBody()) {
      return (
        <Text whiteSpace="pre-wrap">
          {message.getTextBody()}
        </Text>
      )
    }

    return '<NO BODY>'
  }, [message])

  return (
    <Box
      mb="large"
      p="medium"
    >
      <Row alignX="left" mb="medium">
        {message.labels.map(label => (
          <Box key={label} bgColor="yellow" mr="fine">
            {label}
          </Box>
        ))}
        <Spacer />
        <Box>
          {dateFormat.format(message.date)}
        </Box>
      </Row>

      <Box>
        <b>From: </b> {message.from}
      </Box>
      <Box>
        <b>To: </b> {message.to}
      </Box>

      <h3>{message.subject}</h3>

      <Box mb="large">
        {body}
      </Box>
    </Box>
  )
}
