import { cx } from '../../scripts/utils';

type IconProps = {
  variant: 'document-edit' | 'arrow-left' | 'arrow-right' | 'x' | 'search-catalog' | 'add-document';
  className?: string;
};

function throwBadIconVariant(value: never): never;
function throwBadIconVariant(value: IconProps['variant']) {
  throw new Error(`Unknown icon variant: ${value}`);
}

export function Icon({ variant, className = '' }: IconProps) {
  switch (variant) {
    case 'document-edit':
      return (
        <svg className={cx('icon', className)} viewBox="0 0 20 20" fill="currentColor">
          <path d="M13.586 3.586a2 2 0 112.828 2.828l-.793.793-2.828-2.828.793-.793zM11.379 5.793L3 14.172V17h2.828l8.38-8.379-2.83-2.828z" />
        </svg>
      );

    case 'arrow-left':
      // heroicons outline arrow-long-left
      return (
        <svg
          className={cx('icon', className)}
          fill="none"
          viewBox="0 0 24 24"
          stroke="currentColor"
          strokeWidth="1.5"
        >
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            d="M6.75 15.75L3 12m0 0l3.75-3.75M3 12h18"
          />
        </svg>
      );

    case 'arrow-right':
      // heroicons outline arrow-long-right
      return (
        <svg
          className={cx('icon', className)}
          fill="none"
          viewBox="0 0 24 24"
          stroke="currentColor"
          strokeWidth="1.5"
        >
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            d="M17.25 8.25L21 12m0 0l-3.75 3.75M21 12H3"
          />
        </svg>
      );

    case 'x':
      return (
        <svg
          className={cx('icon', className)}
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
        >
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth="2"
            d="M6 18L18 6M6 6l12 12"
          />
        </svg>
      );

    case 'search-catalog':
      return (
        <svg className={cx('icon', className)} viewBox="0 0 20 20" fill="currentColor">
          <path d="M4 4a2 2 0 012-2h4.586A2 2 0 0112 2.586L15.414 6A2 2 0 0116 7.414V16a2 2 0 01-2 2h-1.528A6 6 0 004 9.528V4z" />
          <path
            fillRule="evenodd"
            d="M8 10a4 4 0 00-3.446 6.032l-1.261 1.26a1 1 0 101.414 1.415l1.261-1.261A4 4 0 108 10zm-2 4a2 2 0 114 0 2 2 0 01-4 0z"
            clipRule="evenodd"
          />
        </svg>
      );

    case 'add-document':
      // material design icons file-document-plus-outline
      return (
        <svg className={cx('icon', className)} viewBox="0 0 24 24">
          <path
            fill="currentColor"
            d="M23 18H20V15H18V18H15V20H18V23H20V20H23M6 2C4.89 2 4 2.9 4 4V20C4 21.11 4.89 22 6 22H13.81C13.45 21.38 13.2 20.7 13.08 20H6V4H13V9H18V13.08C18.33 13.03 18.67 13 19 13C19.34 13 19.67 13.03 20 13.08V8L14 2M8 12V14H16V12M8 16V18H13V16Z"
          />
        </svg>
      );
  }

  throwBadIconVariant(variant);
}
