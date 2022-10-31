import { DocumentBackref } from '../../../dto';
import { Ref } from '../../components/Ref';
import { useCardContext } from '../workspace-reducer';

type DocumentViewerBackrefsProps = {
  backrefs: DocumentBackref[];
};
export function DocumentViewerBackrefs({ backrefs }: DocumentViewerBackrefsProps) {
  const { open } = useCardContext();

  return (
    <div className="flex flex-col gap-2 pt-3 border-t-2" hidden={backrefs.length === 0}>
      <h1 className="text-lg font-bold">{backrefs.length} backrefs:</h1>
      {backrefs.map((backref) => (
        <Ref
          key={backref.id}
          documentId={backref.id}
          documentType={backref.documentType}
          subtype={backref.subtype}
          documentTitle={backref.title}
          onClick={() => open({ variant: 'document', documentId: backref.id })}
        />
      ))}
    </div>
  );
}
