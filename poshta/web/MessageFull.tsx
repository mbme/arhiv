import * as React from 'react'
import {
  Text,
  Box,
  Row,
  Spacer,
} from '@v/web-platform'
import { ChronoFormatter } from '@v/chrono'
import { GmailMessage } from './gmail'
import { Sandbox } from './Sandbox'

const dateFormat = new ChronoFormatter('HH:mm DD.MM.YYYY')

interface IProps {
  message: GmailMessage
}

export function MessageFull({ message }: IProps ) {
  const body = React.useMemo(() => {
    if (message.hasHTMLBody()) {
      return (
        <Sandbox content={message.getHTMLBody()} />
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
      px="medium"
      pt="medium"
      mb="medium"
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

      <Box>
        <b>Subject: {message.subject}</b>
      </Box>

      {body}
    </Box>
  )
}
