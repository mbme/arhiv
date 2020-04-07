export interface IGmailProfile {
  emailAddress: string,
  messagesTotal: number,
  threadsTotal: number,
}

interface IGmailMessageHeader {
  name: string,
  value: string,
}

interface IGmailMessagePayloadBody {
  attachmentId?: string,
  data: string, // base64
  size: number,
}

export interface IGmailMessagePayload {
  body: IGmailMessagePayloadBody,
  filename?: string,
  headers: IGmailMessageHeader[],
  mimeType: string,
  partId: string,
  parts?: IGmailMessagePayload[],
}

// https://developers.google.com/gmail/api/v1/reference/users/messages#resource
export interface IGmailMessage {
  id: string,
  internalDate: number,
  labelIds: string[],
  payload: IGmailMessagePayload,
  sizeEstimate: number,
  snippet: string,
  threadId: string,
}

interface IGmailMessageRef {
  id: string,
  threadId: string,
}

export interface IGmailMessageList {
  messages: IGmailMessageRef[],
  nextPageToken: string,
  resultSizeEstimate: number,
}
