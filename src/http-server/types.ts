import http from 'http'
import urlParser from 'url'
import { Stream } from 'stream'

export enum HttpMethod {
  OPTIONS = 'OPTIONS',
  HEAD = 'HEAD',
  GET = 'GET',
  POST = 'POST',
  PUT = 'PUT',
  DELETE = 'DELETE',
  PATCH = 'PATCH',
}
export const parseHttpMethod = (s: string) => {
  switch (s.toUpperCase()) {
    case HttpMethod.OPTIONS.toString():
      return HttpMethod.OPTIONS
    case HttpMethod.HEAD.toString():
      return HttpMethod.HEAD
    case HttpMethod.GET.toString():
      return HttpMethod.GET
    case HttpMethod.POST.toString():
      return HttpMethod.POST
    case HttpMethod.PUT.toString():
      return HttpMethod.PUT
    case HttpMethod.DELETE.toString():
      return HttpMethod.DELETE
    case HttpMethod.PATCH.toString():
      return HttpMethod.PATCH
    default:
      return undefined
  }
}

export interface IHeaders { [name: string]: string }

export interface IRequest {
  url: urlParser.UrlWithParsedQuery,
  method: HttpMethod,
  headers: IHeaders,
  body?: MultipartBody | JSONBody | StringBody
}

export interface IResponse {
  statusCode: number,
  headers: IHeaders,
  body?: string | object | Stream,
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
  constructor(public fields: IMultipartField[] = [], public files: IMultipartFile[] = []) { }

  getField(field: string) {
    return this.fields.find(item => item.field === field)
  }
}
// tslint:disable-next-line:max-classes-per-file
export class JSONBody {
  constructor(public value: object) { }
}
// tslint:disable-next-line:max-classes-per-file
export class StringBody {
  constructor(public value: string) { }
}
