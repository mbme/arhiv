import {
  test,
  assertThrows,
  assertDeepEqual,
  assertMatchSnapshot,
} from '~/tester'
import { command } from './command'
import {
  ArgsParser,
  NeedHelpError,
} from './args-parser'

test('commands support options', () => {
  assertThrows(() => {
    command('test', '').option('TEST', '')
  })
  assertThrows(() => {
    command('test', '').option('test', '')
  })
  assertThrows(() => {
    command('test', '').positional('-test', '')
  })
  assertThrows(() => {
    command('test', '').positionalArray('-test', '')
  })
  assertThrows(() => {
    command('test', '').positionalArray('test', '').option('-x', '')
  })

  assertThrows(() => {
    command('test', '')
      .option('--test', '')
      .parseOptions(['--test', '-ok'])
  })

  const result = command('test', '')
    .option('--test', '')
    .option('-t', '')
    .option('-o', '')
    .positional('test', '')
    .positionalArray('testArr', '')
    .parseOptions(['-t=0', '--test', 'value', '1', '2'])

  assertDeepEqual(result, {
    '--test': '',
    '-t': '0',
    'test': 'value',
    'testArr': ['1', '2'],
  })
})

test('supports commands', () => {
  const p = ArgsParser
    .addCommand(command('test', '').positional('ok', ''))
    .addCommand(command('other', ''))
    .addCommand(command('', '').positional('no', ''))

  assertDeepEqual(p.parse(['test', 'ok']), ['test', { 'ok': 'ok' }])
  assertDeepEqual(p.parse(['other']), ['other', {}])
  assertDeepEqual(p.parse([]), ['', {}])
  assertDeepEqual(p.parse(['no']), ['', { 'no': 'no' }])

  assertThrows(() => {
    ArgsParser
      .addCommand(command('test', ''))
      .parse(['other'])
  })
})

test('supports --help', () => {
  assertThrows(() => {
    ArgsParser
      .addCommand(command('other', ''))
      .parse(['--help'])
  }, NeedHelpError)

  assertThrows(() => {
    ArgsParser
      .withHelp(false)
      .addCommand(command('other', ''))
      .parse(['--help'])
  }, Error)
})

test('mandatory options', () => {
  const p = ArgsParser
    .addCommand(command('test', '')
      .mandatoryOption('--test', '')
      .option('-t', ''))

  assertThrows(() => {
    p.parse(['test', '-t'])
  })

  assertDeepEqual(p.parse(['test', '--test']), ['test', { '--test': '' }])
})

test('options support default values', () => {
  const p = ArgsParser
    .addCommand(command('test', '').option('--port', '', '8080'))

  assertDeepEqual(p.parse(['test']), ['test', { '--port': '8080' }])
})

test('generates help', () => {
  const p = ArgsParser
    .addCommand(
      command('test', 'test command')
        .positional('port', 'port to listen on')
        .mandatoryOption('--option', 'very important option'),
    )
    .addCommand(command('other', 'a different command').positionalArray('args', 'a lot of arguments'))
    .addCommand(command('', 'default command').option('-no', '', 'test'))

  assertMatchSnapshot(p.getHelp('testApp'))
})
