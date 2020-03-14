import {
  test,
  assertEqual,
  assertMatchSnapshot,
} from '@v/tester'
import {
  StyleNode,
  hash2class,
} from './StyleNode'

test('it works', () => {
  const style = new StyleNode({
    fontSize: 1,
  })

  assertEqual(
    style.intoCss()[0],
    `${hash2class(style.hash)} { font-size: 1 }`,
  )
})

test('it generates consistent hashes', () => {
  const obj = {
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
  }

  const style1 = new StyleNode(obj)
  const style2 = new StyleNode(obj)

  assertEqual(style1.hash, style2.hash)
})

test('it supports with nested selectors', () => {
  const style = new StyleNode({
    fontSize: 1,
    '.test &:hover, &': {
      fontSize: 2,
    },
  })

  assertMatchSnapshot(style.intoCss())
})

test('it supports media query', () => {
  const style = new StyleNode({
    fontSize: 1,
    '@media screen and (min-width: 800px)': {
      fontSize: 2,
      '&:hover': {
        margin: 0,
      },
    },
  })

  assertMatchSnapshot(style.intoCss())
})

test('it renders @keyframes', () => {
  const style = new StyleNode({
    from: {
      width: '300%',
      marginLeft: '0',
    },
    to: {
      marginLeft: '100px',
      width: '100%',
    },
  })

  assertMatchSnapshot(style.asKeyframes())
})
