import {
  test,
  assertEqual,
  assertTrue,
  before,
  after,
} from '@v/tester'
import {
  createTempDir,
  rmrfSync,
  readText,
} from '../../fs'
import {
  extractBoundary,
  CRLF,
} from './utils'
import { MultipartParser } from './parser'

let tmpDir: string | undefined
before(async () => {
  tmpDir = await createTempDir()
})
after(() => {
  rmrfSync(tmpDir!)
})

test('extract boundary from Content-Type header', () => {
  const header = 'Content-Type: multipart/form-data; boundary=---------------------------735323031399963166993862150'

  assertEqual(extractBoundary(header), '---------------------------735323031399963166993862150')
})

test('multipart-parser on fake data', async () => {
  const data = `

-----------------------------735323031399963166993862150
Content-Disposition: form-data; name="text1"

text default
-----------------------------735323031399963166993862150
Content-Disposition: form-data; name="text2"

aωb
-----------------------------735323031399963166993862150
Content-Disposition: form-data; name="file1"; filename="a.txt"
Content-Type: text/plain

Content of a.txt.

-----------------------------735323031399963166993862150
Content-Disposition: form-data; name="file2"; filename="a.html"
Content-Type: text/html

<!DOCTYPE html><title>Content of a.html.</title>

-----------------------------735323031399963166993862150
Content-Disposition: form-data; name="file3"; filename="binary"
Content-Type: application/octet-stream

aωb
-----------------------------735323031399963166993862150--
`.replace(/\n/g, CRLF)

  const boundary = '---------------------------735323031399963166993862150'
  const parser = new MultipartParser(boundary, tmpDir!)

  parser.processChunk(Buffer.from(data))

  assertTrue(parser.isComplete())

  const result = parser.getResult()
  assertEqual(result.fields.length, 2)
  assertEqual(result.files.length, 3)

  const fieldText1 = result.getField('text1')
  assertEqual(fieldText1?.value, 'text default')

  const file2 = result.files.find(file => file.field === 'file2')
  assertTrue(!!file2)
  assertEqual(await readText(file2!.file), `<!DOCTYPE html><title>Content of a.html.</title>${CRLF}`)
})

test('multipart-parser on real data', () => {
  const data = `------WebKitFormBoundaryBLSQ8Zcpc0iEjn7h
Content-Disposition: form-data; name="changeset"

{"schemaVersion":1,"baseRev":3,"documents":[],"attachments":[]}
------WebKitFormBoundaryBLSQ8Zcpc0iEjn7h--`.replace(/\n/g, CRLF)

  const boundary = '----WebKitFormBoundaryBLSQ8Zcpc0iEjn7h'
  const parser = new MultipartParser(boundary, tmpDir!)

  parser.processChunk(Buffer.from(data))

  assertTrue(parser.isComplete())

  const result = parser.getResult()
  assertEqual(result.fields.length, 1)
  assertEqual(result.files.length, 0)

  const fieldText1 = result.getField('changeset')
  assertEqual(fieldText1?.value, '{"schemaVersion":1,"baseRev":3,"documents":[],"attachments":[]}')
})
