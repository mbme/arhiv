import { cx } from '../../scripts/utils';

export type IconVariant =
  | 'arrow-left'
  | 'arrow-right'
  | 'x'
  | 'spinner'
  | 'clipboard'
  | 'clipboard-check'
  | 'paperclip'
  | 'folder'
  | 'search'
  | 'search-catalog'
  | 'edit-document'
  | 'add-document'
  | 'erase-document';

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
