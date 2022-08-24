import { DocumentBackref } from '../../dto';
import { Ref } from '../Ref';

type DocumentViewerBackrefsProps = {
  backrefs: DocumentBackref[];
};
export function DocumentViewerBackrefs({ backrefs }: DocumentViewerBackrefsProps) {
  return (
    <div className="flex flex-col gap-2 pt-3 border-t-2" hidden={backrefs.length === 0}>
      <h1 className="text-lg font-bold">{backrefs.length} backrefs:</h1>
      {backrefs.map((backref) => (
        <Ref
          key={backref.id}
          id={backref.id}
          documentType={backref.documentType}
          subtype={backref.subtype}
          title={backref.title}
        />
      ))}
    </div>
  );
}
