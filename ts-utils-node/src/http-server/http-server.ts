/* eslint-disable no-param-reassign */
import http from 'http'
import urlParser from 'url'
import zlib from 'zlib'
import { Stream } from 'stream'
import { Socket } from 'net'
import { createLogger } from '@v/logger'
import {
  isObject,
  promiseTimeout,
  Obj,
} from '@v/utils'
import { PathMatcher } from '@v/utils'
import {
  gzip,
  pipePromise,
} from '../node'
import {
  IContext,
  Next,
  parseHttpMethod,
  IHeaders,
  IRequest,
  IResponse,
  HttpMethod,
} from './types'
import { createBodyParserMiddleware } from './body-parser-middleware'

type Middleware = (context: IContext, next: Next) => Promise<void> | void
type RequestHandler<P extends Obj> = (context: IContext, params: P) => Promise<void> | void

interface IRoute<P extends Obj> {
  test(context: IContext): P | undefined
  cb: RequestHandler<P>
}

const log = createLogger('http-server')

const MAX_SECONDS_TO_WAIT_UNTIL_DESTROY = 10

export class HTTPServer {
  private _middlewares: Middleware[] = []

  private _routes: IRoute<any>[] = []

  private _stopped = false

  constructor(tmpDir: string) {
    this._middlewares.push(createBodyParserMiddleware(tmpDir))
  }

  use(cb: Middleware) {
    this._middlewares.push(cb)
  }

  addRoute<P extends Obj>(
    method: HttpMethod,
    pathMatcher: PathMatcher<P>,
    cb: RequestHandler<P>,
  ) {
    this._routes.push({
      test({ req }: IContext): P | undefined {
        if (req.method.toUpperCase() !== method) {
          return undefined
        }

        return pathMatcher.match(req.url.pathname || '')
      },
      cb,
    })
  }

  get<P extends Obj>(pathMatcher: PathMatcher<P>, cb: RequestHandler<P>) {
    this.addRoute('GET', pathMatcher, cb)
  }

  post<P extends Obj>(pathMatcher: PathMatcher<P>, cb: RequestHandler<P>) {
    this.addRoute('POST', pathMatcher, cb)
  }

  private async _runMiddlewares(context: IContext, pos: number) {
    const middleware = this._middlewares[pos]

    if (!middleware) { // no more middlewares, stop evaluation
      return
    }

    const next = () => this._runMiddlewares(context, pos + 1)
    await Promise.resolve(middleware(context, next))
  }

  private _requestHandler = async (httpReq: http.IncomingMessage, httpRes: http.ServerResponse) => {
    if (this._stopped) {
      log.debug('got a connection while stopping server, ignoring it')

      httpRes.statusCode = 503
      httpRes.end()

      return
    }

    const hrstart = process.hrtime()

    const isGzipSupported = /\bgzip\b/.test((httpReq.headers as IHeaders)['accept-encoding'])

    const req: IRequest = {
      url: urlParser.parse(httpReq.url!, true),
      method: parseHttpMethod(httpReq.method),
      headers: httpReq.headers as IHeaders,
    }

    const res: IResponse = {
      statusCode: 200,
      headers: {},
      body: undefined,
    }

    try {
      await this._runMiddlewares({ req, res, httpReq, httpRes }, 0)
    } catch (e) {
      log.warn('failed to handle request', e)
      res.statusCode = 400
      res.body = { error: e }
    }

    httpRes.statusCode = res.statusCode

    for (const [header, value] of Object.entries(res.headers)) {
      httpRes.setHeader(header, value)
    }

    if (isGzipSupported) {
      httpRes.setHeader('Content-Encoding', 'gzip')
    }

    if (res.body instanceof Stream.Readable) {
      const stream = isGzipSupported ? res.body.pipe(zlib.createGzip()) : res.body
      await pipePromise(stream, httpRes)
    } else if (isObject(res.body)) {
      httpRes.setHeader('Content-Type', 'application/json')
      const str = JSON.stringify(res.body)
      httpRes.end(isGzipSupported ? await gzip(str) : str)
    } else {
      httpRes.end(res.body)
    }

    const hrend = process.hrtime(hrstart)
    const ms = (hrend[0] * 1000) + Math.round(hrend[1] / 1000000)
    log.debug(
      '%s %s %d %s - %dms',
      httpReq.method!.padEnd(4),
      httpReq.url,
      httpRes.statusCode,
      httpRes.statusMessage || 'OK',
      ms,
    )
  }

  // eslint-disable-next-line @typescript-eslint/no-misused-promises
  private _server = http.createServer(this._requestHandler)

  private _sockets = new Set<Socket>()

  start(port: number) {
    if (this._stopped) {
      throw new Error('server has been already stopped')
    }

    // router middleware
    this._middlewares.push(async (context) => {
      for (const route of this._routes) {
        const params = route.test(context)

        if (params) {
          return Promise.resolve(route.cb(context, params))
        }
      }

      context.res.statusCode = 404

      return undefined
    })

    // track open sockets
    this._server.on('connection', (socket) => {
      this._sockets.add(socket)

      socket.once('close', () => {
        this._sockets.delete(socket)
      })
    })

    return new Promise<void>(resolve => this._server.listen(port, resolve))
  }

  private async _stopSockets() {
    let counter = 0

    while (this._sockets.size) {
      if (counter === MAX_SECONDS_TO_WAIT_UNTIL_DESTROY) {
        log.warn(`Destroying ${this._sockets.size} sockets after ${MAX_SECONDS_TO_WAIT_UNTIL_DESTROY} seconds`)

        for (const socket of this._sockets) {
          socket.destroy()
        }

        return
      }

      await promiseTimeout(1000)
      counter += 1
    }
  }

  async stop() {
    if (this._stopped) {
      return
    }

    this._stopped = true

    await Promise.all([
      new Promise<Error | undefined>(resolve => this._server.close(resolve)),
      this._stopSockets(),
    ])
  }
}
