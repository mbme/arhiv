type IconProps = {
  variant: 'document-edit' | 'arrow-left' | 'x';
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
        <svg className={`icon ${className}`} viewBox="0 0 20 20" fill="currentColor">
          <path d="M13.586 3.586a2 2 0 112.828 2.828l-.793.793-2.828-2.828.793-.793zM11.379 5.793L3 14.172V17h2.828l8.38-8.379-2.83-2.828z" />
        </svg>
      );

    case 'arrow-left':
      // heroicons outline arrow-narrow-left
      return (
        <svg
          className={`icon ${className}`}
          fill="none"
          viewBox="0 0 24 24"
          stroke="currentColor"
          strokeWidth="2"
        >
          <path strokeLinecap="round" strokeLinejoin="round" d="M7 16l-4-4m0 0l4-4m-4 4h18" />
        </svg>
      );

    case 'x':
      return (
        <svg className={`icon ${className}`} viewBox="0 0 24 24" fill="none" stroke="currentColor">
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth="2"
            d="M6 18L18 6M6 6l12 12"
          />
        </svg>
      );
  }

  throwBadIconVariant(variant);
}
