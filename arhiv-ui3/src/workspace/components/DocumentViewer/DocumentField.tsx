import { DataDescriptionField } from '../../dto';

type DocumentFieldProps = {
  field: DataDescriptionField;
  value: unknown;
};

export function DocumentField({ field, value }: DocumentFieldProps) {
  return (
    <div className="mb-8">
      {field.name}: {value}
    </div>
  );
}
