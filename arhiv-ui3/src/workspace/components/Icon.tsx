import { ArrayElement, cx } from '../../scripts/utils';

export const ICON_VARIANTS = [
  'arrow-left',
  'arrow-right',
  'x',
  'spinner',
  'clipboard',
  'clipboard-check',
  'paperclip',
  'folder',
  'search',
  'web',
  'more',
  'play',
  'pause',
  'skip-previous',
  'skip-next',
  'volume-off',
  'volume-high',
  'search-catalog',
  'browse-catalog',
  'edit-document',
  'add-document',
  'erase-document',
] as const;

export type IconVariant = ArrayElement<typeof ICON_VARIANTS>;

type IconProps = {
  variant: IconVariant;
  className?: string;
};

function throwBadIconVariant(value: never): never;
function throwBadIconVariant(value: IconVariant) {
  throw new Error(`Unknown icon variant: ${value}`);
}

export function Icon({ variant, className = '' }: IconProps) {
  switch (variant) {
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
            strokeLinecap="round"
            strokeLinejoin="round"
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
    case 'spinner':
      return (
        <svg
          className={cx(
            'icon text-gray-200 animate-spin dark:text-gray-600 fill-blue-600',
            className
          )}
          viewBox="0 0 100 101"
          fill="none"
        >
          <path
            d="M100 50.5908C100 78.2051 77.6142 100.591 50 100.591C22.3858 100.591 0 78.2051 0 50.5908C0 22.9766 22.3858 0.59082 50 0.59082C77.6142 0.59082 100 22.9766 100 50.5908ZM9.08144 50.5908C9.08144 73.1895 27.4013 91.5094 50 91.5094C72.5987 91.5094 90.9186 73.1895 90.9186 50.5908C90.9186 27.9921 72.5987 9.67226 50 9.67226C27.4013 9.67226 9.08144 27.9921 9.08144 50.5908Z"
            fill="currentColor"
          />
          <path
            d="M93.9676 39.0409C96.393 38.4038 97.8624 35.9116 97.0079 33.5539C95.2932 28.8227 92.871 24.3692 89.8167 20.348C85.8452 15.1192 80.8826 10.7238 75.2124 7.41289C69.5422 4.10194 63.2754 1.94025 56.7698 1.05124C51.7666 0.367541 46.6976 0.446843 41.7345 1.27873C39.2613 1.69328 37.813 4.19778 38.4501 6.62326C39.0873 9.04874 41.5694 10.4717 44.0505 10.1071C47.8511 9.54855 51.7191 9.52689 55.5402 10.0491C60.8642 10.7766 65.9928 12.5457 70.6331 15.2552C75.2735 17.9648 79.3347 21.5619 82.5849 25.841C84.9175 28.9121 86.7997 32.2913 88.1811 35.8758C89.083 38.2158 91.5421 39.6781 93.9676 39.0409Z"
            fill="currentFill"
          />
        </svg>
      );

    case 'clipboard':
      // heroicons outline clipboard
      return (
        <svg
          className={cx('icon', className)}
          fill="none"
          viewBox="0 0 24 24"
          strokeWidth="1.5"
          stroke="currentColor"
        >
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            d="M15.666 3.888A2.25 2.25 0 0013.5 2.25h-3c-1.03 0-1.9.693-2.166 1.638m7.332 0c.055.194.084.4.084.612v0a.75.75 0 01-.75.75H9a.75.75 0 01-.75-.75v0c0-.212.03-.418.084-.612m7.332 0c.646.049 1.288.11 1.927.184 1.1.128 1.907 1.077 1.907 2.185V19.5a2.25 2.25 0 01-2.25 2.25H6.75A2.25 2.25 0 014.5 19.5V6.257c0-1.108.806-2.057 1.907-2.185a48.208 48.208 0 011.927-.184"
          />
        </svg>
      );

    case 'clipboard-check':
      // heroicons outline clipboard-document-check
      return (
        <svg
          className={cx('icon', className)}
          fill="none"
          viewBox="0 0 24 24"
          strokeWidth="1.5"
          stroke="currentColor"
        >
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            d="M11.35 3.836c-.065.21-.1.433-.1.664 0 .414.336.75.75.75h4.5a.75.75 0 00.75-.75 2.25 2.25 0 00-.1-.664m-5.8 0A2.251 2.251 0 0113.5 2.25H15c1.012 0 1.867.668 2.15 1.586m-5.8 0c-.376.023-.75.05-1.124.08C9.095 4.01 8.25 4.973 8.25 6.108V8.25m8.9-4.414c.376.023.75.05 1.124.08 1.131.094 1.976 1.057 1.976 2.192V16.5A2.25 2.25 0 0118 18.75h-2.25m-7.5-10.5H4.875c-.621 0-1.125.504-1.125 1.125v11.25c0 .621.504 1.125 1.125 1.125h9.75c.621 0 1.125-.504 1.125-1.125V18.75m-7.5-10.5h6.375c.621 0 1.125.504 1.125 1.125v9.375m-8.25-3l1.5 1.5 3-3.75"
          />
        </svg>
      );
    case 'paperclip':
      // material design icons paperclip-plus
      return (
        <svg className={cx('icon', className)} viewBox="0 0 24 24">
          <path
            fill="currentColor"
            d="M18 13.09C17.47 13.18 16.97 13.34 16.5 13.55V6H18V13.09M12.5 21.5C10.29 21.5 8.5 19.71 8.5 17.5V5C8.5 3.62 9.62 2.5 11 2.5S13.5 3.62 13.5 5V15.5C13.5 16.05 13.05 16.5 12.5 16.5S11.5 16.05 11.5 15.5V6H10V15.5C10 16.88 11.12 18 12.5 18C12.71 18 12.91 17.97 13.1 17.92C13.35 16.58 14.03 15.41 15 14.54V5C15 2.79 13.21 1 11 1S7 2.79 7 5V17.5C7 20.54 9.46 23 12.5 23C13.13 23 13.73 22.89 14.29 22.7C13.97 22.29 13.7 21.84 13.5 21.36C13.17 21.44 12.84 21.5 12.5 21.5M20 18V15H18V18H15V20H18V23H20V20H23V18H20Z"
          />
        </svg>
      );

    case 'folder':
      // heroicons outline folder
      return (
        <svg
          className={cx('icon', className)}
          fill="none"
          viewBox="0 0 24 24"
          strokeWidth={1.5}
          stroke="currentColor"
        >
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            d="M2.25 12.75V12A2.25 2.25 0 014.5 9.75h15A2.25 2.25 0 0121.75 12v.75m-8.69-6.44l-2.12-2.12a1.5 1.5 0 00-1.061-.44H4.5A2.25 2.25 0 002.25 6v12a2.25 2.25 0 002.25 2.25h15A2.25 2.25 0 0021.75 18V9a2.25 2.25 0 00-2.25-2.25h-5.379a1.5 1.5 0 01-1.06-.44z"
          />
        </svg>
      );

    case 'search':
      // heroicons outline magnifying-glass
      return (
        <svg
          className={cx('icon', className)}
          fill="none"
          viewBox="0 0 24 24"
          strokeWidth="1.5"
          stroke="currentColor"
        >
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            d="M21 21l-5.197-5.197m0 0A7.5 7.5 0 105.196 5.196a7.5 7.5 0 0010.607 10.607z"
          />
        </svg>
      );

    case 'web':
      // material design icons web
      return (
        <svg className={cx('icon', className)} viewBox="0 0 24 24">
          <path
            fill="currentColor"
            d="M16.36,14C16.44,13.34 16.5,12.68 16.5,12C16.5,11.32 16.44,10.66 16.36,10H19.74C19.9,10.64 20,11.31 20,12C20,12.69 19.9,13.36 19.74,14M14.59,19.56C15.19,18.45 15.65,17.25 15.97,16H18.92C17.96,17.65 16.43,18.93 14.59,19.56M14.34,14H9.66C9.56,13.34 9.5,12.68 9.5,12C9.5,11.32 9.56,10.65 9.66,10H14.34C14.43,10.65 14.5,11.32 14.5,12C14.5,12.68 14.43,13.34 14.34,14M12,19.96C11.17,18.76 10.5,17.43 10.09,16H13.91C13.5,17.43 12.83,18.76 12,19.96M8,8H5.08C6.03,6.34 7.57,5.06 9.4,4.44C8.8,5.55 8.35,6.75 8,8M5.08,16H8C8.35,17.25 8.8,18.45 9.4,19.56C7.57,18.93 6.03,17.65 5.08,16M4.26,14C4.1,13.36 4,12.69 4,12C4,11.31 4.1,10.64 4.26,10H7.64C7.56,10.66 7.5,11.32 7.5,12C7.5,12.68 7.56,13.34 7.64,14M12,4.03C12.83,5.23 13.5,6.57 13.91,8H10.09C10.5,6.57 11.17,5.23 12,4.03M18.92,8H15.97C15.65,6.75 15.19,5.55 14.59,4.44C16.43,5.07 17.96,6.34 18.92,8M12,2C6.47,2 2,6.5 2,12A10,10 0 0,0 12,22A10,10 0 0,0 22,12A10,10 0 0,0 12,2Z"
          />
        </svg>
      );

    case 'more':
      // material design icons menu
      return (
        <svg className={cx('icon', className)} viewBox="0 0 24 24">
          <path fill="currentColor" d="M3,6H21V8H3V6M3,11H21V13H3V11M3,16H21V18H3V16Z" />
        </svg>
      );

    case 'play':
      // material design icons play
      return (
        <svg className={cx('icon', className)} viewBox="0 0 24 24">
          <path fill="currentColor" d="M8,5.14V19.14L19,12.14L8,5.14Z" />
        </svg>
      );

    case 'pause':
      // material design icons pause
      return (
        <svg className={cx('icon', className)} viewBox="0 0 24 24">
          <path fill="currentColor" d="M14,19H18V5H14M6,19H10V5H6V19Z" />
        </svg>
      );

    case 'skip-previous':
      // material design icons skip-previous
      return (
        <svg className={cx('icon', className)} viewBox="0 0 24 24">
          <path fill="currentColor" d="M6,18V6H8V18H6M9.5,12L18,6V18L9.5,12Z" />
        </svg>
      );

    case 'skip-next':
      // material design icons skip-next
      return (
        <svg className={cx('icon', className)} viewBox="0 0 24 24">
          <path fill="currentColor" d="M16,18H18V6H16M6,18L14.5,12L6,6V18Z" />
        </svg>
      );

    case 'volume-off':
      // material design icons volume-off
      return (
        <svg className={cx('icon', className)} viewBox="0 0 24 24">
          <path
            fill="currentColor"
            d="M12,4L9.91,6.09L12,8.18M4.27,3L3,4.27L7.73,9H3V15H7L12,20V13.27L16.25,17.53C15.58,18.04 14.83,18.46 14,18.7V20.77C15.38,20.45 16.63,19.82 17.68,18.96L19.73,21L21,19.73L12,10.73M19,12C19,12.94 18.8,13.82 18.46,14.64L19.97,16.15C20.62,14.91 21,13.5 21,12C21,7.72 18,4.14 14,3.23V5.29C16.89,6.15 19,8.83 19,12M16.5,12C16.5,10.23 15.5,8.71 14,7.97V10.18L16.45,12.63C16.5,12.43 16.5,12.21 16.5,12Z"
          />
        </svg>
      );

    case 'volume-high':
      // material design icons volume-high
      return (
        <svg className={cx('icon', className)} viewBox="0 0 24 24">
          <path
            fill="currentColor"
            d="M14,3.23V5.29C16.89,6.15 19,8.83 19,12C19,15.17 16.89,17.84 14,18.7V20.77C18,19.86 21,16.28 21,12C21,7.72 18,4.14 14,3.23M16.5,12C16.5,10.23 15.5,8.71 14,7.97V16C15.5,15.29 16.5,13.76 16.5,12M3,9V15H7L12,20V4L7,9H3Z"
          />
        </svg>
      );

    case 'search-catalog':
      // material design icons card-search-outline
      return (
        <svg className={cx('icon', className)} viewBox="0 0 24 24">
          <path
            fill="currentColor"
            d="M11.5 16C12.37 16 13.19 15.74 13.88 15.3L16.32 17.74L17.74 16.32L15.3 13.89C15.74 13.19 16 12.38 16 11.5C16 9 14 7 11.5 7S7 9 7 11.5 9 16 11.5 16M11.5 9C12.88 9 14 10.12 14 11.5S12.88 14 11.5 14 9 12.88 9 11.5 10.12 9 11.5 9M20 4H4C2.9 4 2 4.9 2 6V18C2 19.1 2.9 20 4 20H20C21.1 20 22 19.1 22 18V6C22 4.9 21.1 4 20 4M20 18H4V6H20V18Z"
          />
        </svg>
      );

    case 'browse-catalog':
      // heroicons outline inbox-stack
      return (
        <svg
          className={cx('icon', className)}
          fill="none"
          viewBox="0 0 24 24"
          stroke-width="1.5"
          stroke="currentColor"
        >
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            d="M7.875 14.25l1.214 1.942a2.25 2.25 0 001.908 1.058h2.006c.776 0 1.497-.4 1.908-1.058l1.214-1.942M2.41 9h4.636a2.25 2.25 0 011.872 1.002l.164.246a2.25 2.25 0 001.872 1.002h2.092a2.25 2.25 0 001.872-1.002l.164-.246A2.25 2.25 0 0116.954 9h4.636M2.41 9a2.25 2.25 0 00-.16.832V12a2.25 2.25 0 002.25 2.25h15A2.25 2.25 0 0021.75 12V9.832c0-.287-.055-.57-.16-.832M2.41 9a2.25 2.25 0 01.382-.632l3.285-3.832a2.25 2.25 0 011.708-.786h8.43c.657 0 1.281.287 1.709.786l3.284 3.832c.163.19.291.404.382.632M4.5 20.25h15A2.25 2.25 0 0021.75 18v-2.625c0-.621-.504-1.125-1.125-1.125H3.375c-.621 0-1.125.504-1.125 1.125V18a2.25 2.25 0 002.25 2.25z"
          />
        </svg>
      );

    case 'edit-document':
      return (
        <svg className={cx('icon', className)} viewBox="0 0 20 20" fill="currentColor">
          <path d="M13.586 3.586a2 2 0 112.828 2.828l-.793.793-2.828-2.828.793-.793zM11.379 5.793L3 14.172V17h2.828l8.38-8.379-2.83-2.828z" />
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

    case 'erase-document':
      // material design icons file-document-remove-outline
      return (
        <svg className={cx('icon', className)} viewBox="0 0 24 24">
          <path
            fill="currentColor"
            d="M22.54 21.12L20.41 19L22.54 16.88L21.12 15.46L19 17.59L16.88 15.46L15.46 16.88L17.59 19L15.46 21.12L16.88 22.54L19 20.41L21.12 22.54M6 2C4.89 2 4 2.9 4 4V20C4 21.11 4.89 22 6 22H13.81C13.45 21.38 13.2 20.7 13.08 20H6V4H13V9H18V13.08C18.33 13.03 18.67 13 19 13C19.34 13 19.67 13.03 20 13.08V8L14 2M8 12V14H16V12M8 16V18H13V16Z"
          />
        </svg>
      );
  }

  throwBadIconVariant(variant);
}
