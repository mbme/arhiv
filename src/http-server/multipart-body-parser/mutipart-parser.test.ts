import {
  test,
  assertEqual,
  assertTrue,
  before,
  after,
} from '~/tester'
import {
  createTempDir,
  rmrfSync,
  readText,
} from '~/utils/fs'
import { extractBoundary } from './utils'
import { MultipartParser } from './parser'

let tmpDir: string | undefined
before(async () => {
  tmpDir = await createTempDir()
})
after(() => {
  rmrfSync(tmpDir!)
})

test('extract boundary from Content-Type header', () => {
  assertEqual(
    extractBoundary('Content-Type: multipart/form-data; boundary=---------------------------735323031399963166993862150'),
    '---------------------------735323031399963166993862150',
  )
})

test('multipart-parser', async () => {
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
`

  const boundary = '---------------------------735323031399963166993862150'
  const parser = new MultipartParser(boundary, tmpDir!)

  parser.processChunk(Buffer.from(data))

  assertTrue(parser.isComplete())

  const result = parser.getResult()
  assertEqual(result.fields.length, 2)
  assertEqual(result.files.length, 3)

  const fieldText1 = result.fields.find(field => field.field === 'text1')
  assertEqual(fieldText1?.value, 'text default')

  const file2 = result.files.find(file => file.field === 'file2')
  assertTrue(!!file2)
  assertEqual(await readText(file2!.file), '<!DOCTYPE html><title>Content of a.html.</title>\n')
})
