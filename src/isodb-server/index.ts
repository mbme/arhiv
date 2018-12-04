import path from 'path';
import fs from 'fs';
import { rmrfSync, createTempDir } from '../fs/utils';
import { lazy } from '../utils';
import createQueue from '../utils/queue';
import { getMimeType } from '../file-prober';
import Server from '../http-server';
import {
  isValidAuth,
  extractToken,
  resolveAsset,
  readFormData,
} from './utils';
import PrimaryDB from '../isodb/primary';

const STATIC_DIR = path.join(__dirname, '../client/static');
const DIST_DIR = path.join(__dirname, '../../dist');

export default function createServer(db: PrimaryDB, password = '') {
  const queue = createQueue()
  const server = new Server();

  server.use(async function bootstrapMiddleware(context, next) {
    const { req, res } = context;

    res.headers['Referrer-Policy'] = 'no-referrer'

    const isAuthorized = isValidAuth(extractToken(req.headers.cookie || ''), password);
    if (req.url.pathname!.startsWith('/api') && !isAuthorized) {
      res.statusCode = 403
      return;
    }

    await next();
  });

  server.post('/api/changes', async ({ res, req }) => {
    const isMultipartRequest = (req.headers['content-type'] || '').startsWith('multipart/form-data');
    if (!isMultipartRequest) {
      res.statusCode = 415
      return;
    }

    const tmpDir = lazy(createTempDir)
    const {
      data,
      assets,
    } = await readFormData(tmpDir, req);

    try {
      const rev = parseInt(data.rev, 10);
      const records = JSON.parse(data.records);

      const success = await queue.push(async () => db.applyChanges(rev, records, assets));
      res.statusCode = success ? 200 : 409
      // FIXME send patch in response
    } finally {
      if (tmpDir.initialized) rmrfSync(await tmpDir.value);
    }
  });

  server.get('/api/patch', async ({ req, res }) => {
    const rev = req.url.query.rev as string;
    if (!rev) {
      res.statusCode = 400
      return;
    }

    const patch = await queue.push(async () => db.getPatch(parseInt(rev, 10)));
    res.body = patch
  });

  server.get('/api', async ({ req, res }) => {
    const fileId = req.url.query.fileId as string;
    if (!fileId) {
      res.statusCode = 400
      return;
    }

    const filePath = await queue.push(async () => db.getAttachment(fileId));

    if (filePath) {
      res.headers['Content-Disposition'] = `inline; filename=${fileId}`
      res.headers['Content-Type'] = await getMimeType(filePath)
      res.headers['Cache-Control'] = 'immutable, private, max-age=31536000' // max caching
      res.body = fs.createReadStream(filePath)
    } else {
      res.statusCode = 404
    }
  });

  // Handle assets + html5 history fallback
  server.get(() => true, async ({ req, res }) => {
    const fileName = req.url.path!.substring(1); // skip leading /
    const filePath = await resolveAsset(STATIC_DIR, fileName)
      || await resolveAsset(DIST_DIR, fileName)
      || await resolveAsset(STATIC_DIR, 'index.html'); // html5 history fallback

    if (filePath) {
      res.headers['Content-Type'] = await getMimeType(filePath)
      res.body = fs.createReadStream(filePath)
    } else {
      res.statusCode = 404
    }
  });

  return {
    start(port: number) {
      return server.start(port)
    },

    stop() {
      return Promise.all([server.stop(), queue.close()]);
    },
  }
}
