import http from 'http'
import urlParser from 'url'
import { Stream } from 'stream'
import { Obj } from '@v/utils'

const HTTP_METHOD = {
  OPTIONS: 'OPTIONS',
  HEAD: 'HEAD',
  GET: 'GET',
  POST: 'POST',
  PUT: 'PUT',
  DELETE: 'DELETE',
  PATCH: 'PATCH',
}
export type HttpMethod = keyof typeof HTTP_METHOD

export function parseHttpMethod(s?: string): HttpMethod {
  if (!s) {
    throw new Error('http method is missing')
  }

  const normalizedS = s.toUpperCase()

  if (!(HTTP_METHOD as Obj)[normalizedS]) {
    throw new Error(`got unexpected http method ${s}`)
  }

  return normalizedS as HttpMethod
}

export interface IHeaders {
  [name: string]: string
}

export interface IRequest {
  url: urlParser.UrlWithParsedQuery,
  method: HttpMethod,
  headers: IHeaders,
  body?: MultipartBody | JSONBody | StringBody
}

export interface IResponse {
  statusCode: number,
  headers: IHeaders,
  body?: string | Obj | Stream,
}

export type Next = () => Promise<void>
export interface IContext {
  req: IRequest,
  res: IResponse,
  httpReq: http.IncomingMessage,
  httpRes: http.ServerResponse,
}

interface IMultipartField {
  field: string
  value: string
}
interface IMultipartFile {
  field: string
  file: string
}

export class MultipartBody {
  constructor(
    public readonly fields: IMultipartField[] = [],
    public readonly files: IMultipartFile[] = [],
  ) { }

  getField(field: string) {
    return this.fields.find(item => item.field === field)
  }
}

export class JSONBody {
  constructor(public readonly value: Obj) { }
}

export class StringBody {
  constructor(public readonly value: string) { }
}
