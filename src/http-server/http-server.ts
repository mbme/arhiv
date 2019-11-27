import http from 'http'
import urlParser from 'url'
import zlib from 'zlib'
import { Stream } from 'stream'
import { Socket } from 'net'
import { createLogger } from '~/logger'
import {
  isString,
  isObject,
  promiseTimeout,
} from '~/utils'
import {
  gzip,
  pipePromise,
} from '~/utils/node'
import {
  IContext,
  Next,
  HttpMethod,
  parseHttpMethod,
  IHeaders,
  IRequest,
  IResponse,
} from './types'
import { bodyParserMiddleware } from './body-parser-middleware'

type Middleware = (context: IContext, next: Next) => Promise<void> | void
type RequestHandler = (context: IContext) => Promise<void> | void
type PathTest = ((path: string) => boolean) | string

interface IRoute {
  test(context: IContext): boolean
  cb: RequestHandler
}

const log = createLogger('http-server')

const MAX_SECONDS_TO_WAIT_UNTIL_DESTROY = 10

export class HTTPServer {
  private _middlewares: Middleware[] = [bodyParserMiddleware]
  private _routes: IRoute[] = []
  private _stopped = false

  use(cb: Middleware) {
    this._middlewares.push(cb)
  }

  addRoute(method: HttpMethod, pathTest: PathTest, cb: RequestHandler) {
    this._routes.push({
      test({ req }: IContext) {
        if (req.method !== method.toString()) return false
        if (isString(pathTest)) return req.url.pathname === pathTest

        return pathTest(req.url.pathname!)
      },
      cb,
    })
  }

  get(pathTest: PathTest, cb: RequestHandler) {
    this.addRoute(HttpMethod.GET, pathTest, cb)
  }

  post(pathTest: PathTest, cb: RequestHandler) {
    this.addRoute(HttpMethod.POST, pathTest, cb)
  }

  private async _runMiddlewares(context: IContext, pos: number) {
    const middleware = this._middlewares[pos]

    if (!middleware) return // no more middlewares, stop evaluation

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
      method: parseHttpMethod(httpReq.method!)!,
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

  private _server = http.createServer(this._requestHandler)
  private _sockets = new Set<Socket>()

  start(port: number) {
    if (this._stopped) {
      throw new Error('server has been already stopped')
    }

    // router middleware
    this._middlewares.push(async (context) => {
      const route = this._routes.find((item) => item.test(context))

      if (route) {
        await Promise.resolve(route.cb(context))
      } else {
        context.res.statusCode = 404
      }
    })

    // track open sockets
    this._server.on('connection', (socket) => {
      this._sockets.add(socket)

      socket.once('close', () => {
        this._sockets.delete(socket)
      })
    })

    return new Promise<void>((resolve) => this._server.listen(port, resolve))
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
      new Promise<Error | undefined>((resolve) => this._server.close(resolve)),
      this._stopSockets(),
    ])
  }
}
