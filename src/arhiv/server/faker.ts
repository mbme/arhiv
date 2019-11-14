import path from 'path'
import fs from 'fs'
import { getMimeType } from '~/file-prober'
import {
  createArray,
  IDict,
  nowS,
} from '~/utils'
import {
  readText,
  listFiles,
  getFileSize,
  createTempDir,
} from '~/utils/fs'
import { createLink } from '~/markup-parser/utils'
import {
  randomInt,
  shuffle,
  pickRandomItem,
} from '~/utils/random'
import createTextGenerator, { ITextGenerator } from '~/utils/random/text-generator'
import {
  generateRandomId,
  createDocument,
} from '../isodb/utils'
import { IAttachment } from '../isodb/types'
import {
  DocumentType,
  INote,
} from '../types'

async function getFakeNote(generator: ITextGenerator, images: IDict): Promise<INote> {
  const name = generator.sentence(1, 8)

  const refs = new Set<string>()

  const data = createArray(
    randomInt(1, 7), // paragraphs
    () => {
      const sentences = createArray(
        randomInt(1, 7), // sentences
        () => generator.sentence(),
      )

      if (Math.random() < 0.34) {
        const imageId = pickRandomItem(Object.keys(images))
        refs.add(imageId)

        const link = createLink(imageId, path.basename(images[imageId]))
        sentences.push(` ${link} `)
      }

      return shuffle(sentences).join(' ')
    },
  ).join('\n\n')

  return {
    ...createDocument(generateRandomId(), DocumentType.Note),
    _attachmentRefs: Array.from(refs),
    name: name.substring(0, name.length - 1),
    data,
  }
}

async function prepareImages(basePath: string, tempDir: string): Promise<IDict> {
  const files = await listFiles(basePath)
  const images = files.filter((name) => name.match(/\.(jpg|jpeg)$/i))

  const result: IDict = {}

  await Promise.all(images.map(async (name) => {
    const filePath = path.join(basePath, name)
    const imageId = generateRandomId()

    const newPath = path.join(tempDir, name)

    await fs.promises.copyFile(filePath, newPath)

    result[imageId] = newPath
  }))

  return result
}

async function createAttachments(images: IDict): Promise<IAttachment[]> {
  return Promise.all(Object.entries(images).map(async ([_id, imagePath]) => {
    const [
      _mimeType,
      _size,
    ] = await Promise.all([
      getMimeType(imagePath),
      getFileSize(imagePath),
    ])

    return {
      _id,
      _rev: 0,
      _createdTs: nowS(),
      _mimeType,
      _size,
    }
  }))
}

export async function getFakeNotes(appDir: string, count: number) {
  const resourcesPath = path.join(appDir, 'resources')
  const tempDir = await createTempDir()
  const images = await prepareImages(resourcesPath, tempDir)

  const text = await readText(path.join(resourcesPath, 'text.txt'))
  const generator = createTextGenerator(text)

  return {
    documents: await Promise.all(createArray(count, () => getFakeNote(generator, images))),
    attachments: await createAttachments(images),
    attachedFiles: images,
    tempDir,
  }
}
