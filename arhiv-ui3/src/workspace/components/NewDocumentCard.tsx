import { Card } from './Card';

type NewDocumentCardProps = {
  documentType: string;
};
export function NewDocumentCard({ documentType }: NewDocumentCardProps) {
  return (
    <Card>
      <h1>NEW DOCUMENT</h1>
      <h2>{documentType}</h2>
    </Card>
  );
}
