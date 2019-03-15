import {
  style,
  keyframes,
} from 'typestyle'
import theme from './theme'

type Spacing = keyof typeof theme.spacing
interface ISpacingProps {
  left?: Spacing
  right?: Spacing
  top?: Spacing
  bottom?: Spacing
  horizontal?: Spacing
  vertical?: Spacing
  all?: Spacing
}

const spacing = (prop: string) => ({ left, right, top, bottom, horizontal, vertical, all }: ISpacingProps) => style(
  left && {
    [prop + 'Left']: theme.spacing[left],
  },
  right && {
    [prop + 'Right']: theme.spacing[right],
  },
  top && {
    [prop + 'Top']: theme.spacing[top],
  },
  bottom && {
    [prop + 'Bottom']: theme.spacing[bottom],
  },
  horizontal && {
    [prop + 'Left']: theme.spacing[horizontal],
    [prop + 'Right']: theme.spacing[horizontal],
  },
  vertical && {
    [prop + 'Top']: theme.spacing[vertical],
    [prop + 'Bottom']: theme.spacing[vertical],
  },
  all && {
    [prop]: theme.spacing[all],
  },
)

export const margin = spacing('margin')
export const padding = spacing('padding')

export const section = margin({ bottom: 'medium' })

export const flexRow = (justifyContent: string = 'center') => style({
  display: 'flex',
  flexWrap: 'wrap',
  alignItems: 'center',
  justifyContent,
})

export const animation = {
  pulse: keyframes({
    '0%': {
      opacity: 0.7,
    },

    '50%': {
      opacity: 1,
    },

    '100%': {
      opacity: 0.7,
    },
  }),

  spin: keyframes({
    from: {
      transform: 'rotate(0deg)',
    },

    to: {
      transform: 'rotate(359deg)',
    },
  }),
}
