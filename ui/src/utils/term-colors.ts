// based on https://blog.bitsrc.io/coloring-your-terminal-using-nodejs-eb647d4af2a2

function code2string(code: number) {
  return `\x1b[${code}m`
}

// const globalReset = code2string(0)

export type ColorCode = (text: string) => string

function createColorCode(code: number, resetCode: number): ColorCode {
  return (text: string) => [
    code2string(code),
    text,
    code2string(resetCode),
  ].join('')
}

export const termColors = {
  // text colors
  blue: createColorCode(34, 39),
  yellow: createColorCode(33, 39),
  red: createColorCode(31, 39),
  cyan: createColorCode(36, 39),
  green: createColorCode(32, 39),
  magenta: createColorCode(35, 39),
  white: createColorCode(37, 39),
  gray: createColorCode(30, 39),
  redBright: createColorCode(91, 39),
  greenBright: createColorCode(92, 39),
  yellowBright: createColorCode(93, 39),
  blueBright: createColorCode(94, 39),
  magentaBright: createColorCode(95, 39),
  cyanBright: createColorCode(96, 39),
  whiteBright: createColorCode(97, 39),

  // background colors
  bgBlack: createColorCode(40, 49),
  bgRed: createColorCode(41, 49),
  bgGreen: createColorCode(42, 49),
  bgYellow: createColorCode(43, 49),
  bgBlue: createColorCode(44, 49),
  bgMagenta: createColorCode(45, 49),
  bgCyan: createColorCode(46, 49),
  bgWhite: createColorCode(47, 49),
  bgBlackBright: createColorCode(100, 49),
  bgRedBright: createColorCode(101, 49),
  bgGreenBright: createColorCode(102, 49),
  bgYellowBright: createColorCode(103, 49),
  bgBlueBright: createColorCode(104, 49),
  bgMagentaBright: createColorCode(105, 49),
  bgCyanBright: createColorCode(106, 49),
  bgWhiteBright: createColorCode(107, 49),

  // styling
  bold: createColorCode(1, 22),
  dim: createColorCode(2, 22),
  italic: createColorCode(3, 23),
  underline: createColorCode(4, 24),
  inverse: createColorCode(7, 27),
  hidden: createColorCode(8, 28),
  strikethrough: createColorCode(9, 29),
}

export type TermColor = keyof typeof termColors
