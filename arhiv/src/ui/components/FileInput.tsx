import { cx } from 'utils';
import { ButtonVariant, getButtonVariantClasses } from 'components/Button';

type Props = {
  label: string;
  variant: ButtonVariant;
  className?: string;
  required?: boolean;
  disabled?: boolean;
} & (
  | {
      multiple: true;
      onSelected: (files: File[]) => void;
    }
  | {
      multiple?: false;
      onSelected: (file?: File) => void;
    }
);

export function FileInput({
  label,
  className,
  onSelected,
  variant,
  required,
  disabled,
  multiple,
}: Props) {
  return (
    <label className={cx(getButtonVariantClasses(variant), className)}>
      {label}

      <input
        className="hidden"
        type="file"
        required={required}
        multiple={multiple}
        disabled={disabled}
        onChange={(e) => {
          const files = [...(e.currentTarget.files ?? [])];

          if (multiple) {
            onSelected(files);
          } else {
            onSelected(files[0]);
          }
        }}
      />
    </label>
  );
}
