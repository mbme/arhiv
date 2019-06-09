import { test } from '~/tester'
import { Style } from './Style'
import { hash2class } from './utils'

test('Style works', (assert) => {
  const style = new Style({
    fontSize: 1,
  })

  assert.equal(
    style.intoCss()[0],
    `${hash2class(style.hash)} { font-size: 1 }`,
  )
})

test('Style generates consistent hashes', (assert) => {
  const style1 = new Style({
    fontSize: 1,
    '@media': {
      fontSize: 1,
      margin: 0,
    },
    '&:hover': {
      fontSize: 1,
      margin: 0,
    },
    margin: 0,
  })

  const style2 = new Style({
    margin: 0,
    '@media': {
      margin: 0,
      fontSize: 1,
    },
    fontSize: 1,
    '&:hover': {
      margin: 0,
      fontSize: 1,
    },
  })

  assert.equal(style1.hash, style2.hash)
})

test('Style with nested selectors', (assert) => {
  const style = new Style({
    fontSize: 1,
    '.test &:hover': {
      fontSize: 2,
    },
  })

  assert.matchSnapshot(style.intoCss())
})

test('style with media query', (assert) => {
  const style = new Style({
    fontSize: 1,
    '@media screen and (min-width: 800px)': {
      fontSize: 2,
      '&:hover': {
        margin: 0,
      },
    },
  })

  assert.matchSnapshot(style.intoCss())
})
