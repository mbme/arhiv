// based on https://blog.bitsrc.io/coloring-your-terminal-using-nodejs-eb647d4af2a2

function code2string(code: number) {
  return `\x1b[${code}m`
}

const reset = code2string(0)

type ColorCode = (text: string, withReset?: boolean) => string

function createColorCode(code0: number, code1: number): ColorCode {
  return (text: string, withReset = true) => [
    code2string(code0),
    text,
    code2string(code1),
    withReset ? reset : '',
  ].join('')
}

const fg = {
  blue: createColorCode(34, 89),
  yellow: createColorCode(33, 89),
  red: createColorCode(31, 89),
  cyan: createColorCode(36, 89),
  green: createColorCode(32, 89),
  magenta: createColorCode(35, 89),
  white: createColorCode(37, 89),
  gray: createColorCode(30, 89),
  redBright: createColorCode(91, 39),
  greenBright: createColorCode(92, 39),
  yellowBright: createColorCode(93, 39),
  blueBright: createColorCode(94, 39),
  magentaBright: createColorCode(95, 39),
  cyanBright: createColorCode(96, 39),
  whiteBright: createColorCode(97, 39),
}

const bg = {
  black: createColorCode(40, 49),
  red: createColorCode(41, 49),
  green: createColorCode(42, 49),
  yellow: createColorCode(43, 49),
  blue: createColorCode(44, 49),
  magenta: createColorCode(45, 49),
  cyan: createColorCode(46, 49),
  white: createColorCode(47, 49),
  blackBright: createColorCode(100, 49),
  redBright: createColorCode(101, 49),
  greenBright: createColorCode(102, 49),
  yellowBright: createColorCode(103, 49),
  blueBright: createColorCode(104, 49),
  magentaBright: createColorCode(105, 49),
  cyanBright: createColorCode(106, 49),
  whiteBright: createColorCode(107, 49),
}

const styling = {
  bold: createColorCode(1, 22),
  dim: createColorCode(2, 22),
  italic: createColorCode(3, 23),
  underline: createColorCode(4, 24),
  inverse: createColorCode(7, 27),
  hidden: createColorCode(8, 28),
  strikethrough: createColorCode(9, 29),
}

export const termColors = {
  fg,
  bg,
  styling,
  reset,
}
