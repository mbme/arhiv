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
  let body: React.ReactNode = '<NO BODY>'

  if (message.hasHTMLBody()) {
    body = (
      <Box
        dangerouslySetInnerHTML={{ __html: message.getHTMLBody() }}
      />
    )
  } else if (message.hasTextBody()) {
    body = (
      <Text>
        {message.getTextBody()}
      </Text>
    )
  }

  return (
    <Box
      mb="large"
      p="medium"
    >
      <Row alignX="left">
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

      <h3>{message.subject}</h3>

      <Box mb="large">
        {body}
      </Box>

      <Box>
        <b>From: </b> {message.from}
      </Box>
      <Box>
        <b>To: </b> {message.to}
      </Box>
    </Box>
  )
}
