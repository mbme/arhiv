import http from 'http'
import urlParser from 'url'
import log from '../logger'
import { isString } from '../utils'
import { Stream } from 'stream';

export enum HttpMethod {
  OPTIONS = 'OPTIONS',
  HEAD = 'HEAD',
  GET = 'GET',
  POST = 'POST',
  PUT = 'PUT',
  DELETE = 'DELETE',
  PATCH = 'PATCH',
}

export type Headers = { [name: string]: string }

export type Request = {
  url: urlParser.UrlWithParsedQuery,
  method: HttpMethod,
  headers: Headers,
}

export type Response = {
  statusCode: number,
  headers: Headers,
  body?: string | object | Stream,
}

type Next = () => Promise<void>
interface IContext {
  req: Request,
  res: Response,
}
type Middleware = (context: IContext, next: Next) => Promise<void> | void
type RequestHandler = (context: IContext) => Promise<void> | void
type PathTest = ((path: string) => boolean) | string

interface IRoute {
  test: (context: IContext) => boolean
  cb: RequestHandler
}

async function runMiddlewares(middlewares: Middleware[], context: IContext, pos: number) {
  const middleware = middlewares[pos]

  if (!middleware) return // no more middlewares, stop evaluation

  const next = () => runMiddlewares(middlewares, context, pos + 1)
  await Promise.resolve(middleware(context, next))
}

async function loggerMiddleware({ req, res }: IContext, next: Next) {
  const hrstart = process.hrtime()
  try {
    await next()
  } finally {
    const hrend = process.hrtime(hrstart)
    const ms = (hrend[0] * 1000) + Math.round(hrend[1] / 1000000)
    log.debug('%s %s %d %s - %dms', req.method.padEnd(4), req.url, res.statusCode, res.statusMessage || 'OK', ms)
  }
}

export default class Server {
  _server: http.Server | undefined

  _middlewares: Middleware[] = [loggerMiddleware]
  _routes: IRoute[] = []

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

  start(port: number) {
    // router middleware
    this._middlewares.push(async (context) => {
      const route = this._routes.find((item) => item.test(context))

      if (route) {
        await Promise.resolve(route.cb(context))
      } else {
        context.res.statusCode = 404
      }
    })

    this._server = http.createServer(async (httpReq: http.IncomingMessage, httpRes: http.ServerResponse) => {
      const isGzipSupported = /\bgzip\b/.test((httpReq.headers as Headers)['accept-encoding']);
      const req: Request = {
        url: urlParser.parse(httpReq.url!, true),
        method: HttpMethod[httpReq.method!],
        headers: httpReq.headers as Headers,
      };
      const res: Response = {
        statusCode: 200,
        headers: {},
        body: undefined,
      };
      try {
        await runMiddlewares(this._middlewares, { req, res }, 0)
      } catch (e) {
        log.warn('failed to handle request', e)
        res.statusCode = 400
        res.body = { error: e.toString() };
      }

      if (isGzipSupported) {
        res.headers['Content-Encoding'] = 'gzip'
      }

      res.writeHead(400, { 'Content-Type': 'application/json' })
      res.end(JSON.stringify({ error: e.toString() }))
    })

    return new Promise((resolve) => this._server!.listen(port, resolve))
  }

  stop() {
    return new Promise((resolve) => this._server!.close(resolve))
  }
}
