import path from 'path'
import fs from 'fs'
import { getMimeType } from '~/file-prober'
import {
  createArray,
  Dict,
  dateNow,
} from '~/utils'
import {
  readText,
  listFiles,
  getFileSize,
  createTempDir,
} from '~/utils/fs'
import {
  createLink,
} from '~/markup-parser/utils'
import {
  randomInt,
  shuffle,
  pickRandomItem,
} from '~/utils/random'
import {
  ITextGenerator,
  createTextGenerator,
} from '~/utils/random/text-generator'
import {
  generateRandomId,
  createDocument,
} from '../utils'
import { IDocument, IAttachment } from '../schema'
import { INoteProps } from '../replica/entities/note-manager'

async function getFakeNote(generator: ITextGenerator, images: Dict): Promise<IDocument<'note', INoteProps>> {
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
    ...createDocument(generateRandomId(), 'note', {
      name: name.substring(0, name.length - 1),
      data,
    }),
    attachmentRefs: Array.from(refs),
  }
}

async function prepareImages(basePath: string, tempDir: string): Promise<Dict> {
  const files = await listFiles(basePath)
  const images = files.filter((name) => name.match(/\.(jpg|jpeg)$/i))

  const result: Dict = {}

  await Promise.all(images.map(async (name) => {
    const filePath = path.join(basePath, name)
    const imageId = generateRandomId()

    const newPath = path.join(tempDir, name)

    await fs.promises.copyFile(filePath, newPath)

    result[imageId] = newPath
  }))

  return result
}

async function createAttachments(images: Dict): Promise<IAttachment[]> {
  return Promise.all(Object.entries(images).map(async ([id, imagePath]) => {
    const [
      mimeType,
      size,
    ] = await Promise.all([
      getMimeType(imagePath),
      getFileSize(imagePath),
    ])

    return {
      id,
      rev: 0,
      createdAt: dateNow(),
      mimeType,
      size,
      deleted: false,
    }
  }))
}

export async function getFakeNotes(resourcesDir: string, count: number) {
  const tempDir = await createTempDir()
  const images = await prepareImages(resourcesDir, tempDir)

  const text = await readText(path.join(resourcesDir, 'text.txt'))
  const generator = createTextGenerator(text)

  return {
    documents: await Promise.all(createArray(count, () => getFakeNote(generator, images))),
    attachments: await createAttachments(images),
    attachedFiles: images,
    tempDir,
  }
}
