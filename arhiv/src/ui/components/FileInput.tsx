import { cx } from 'utils';
import { ButtonVariant, getButtonVariantClasses } from 'components/Button';

interface Props {
  label: string;
  onFileSelected: (file?: File) => void;
  variant: ButtonVariant;
  className?: string;
  required?: boolean;
}

export function FileInput({ label, className, onFileSelected, variant, required }: Props) {
  return (
    <label className={cx(getButtonVariantClasses(variant), className)}>
      {label}

      <input
        className="hidden"
        type="file"
        required={required}
        onChange={(e) => {
          onFileSelected(e.currentTarget.files?.[0] ?? undefined);
        }}
      />
    </label>
  );
}
