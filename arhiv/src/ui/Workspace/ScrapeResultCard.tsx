import { DocumentId } from 'dto';
import { useSuspenseQuery } from 'utils/suspense';
import { Link } from 'components/Link';
import { Ref } from 'components/Ref';
import { CardContainer } from './CardContainer';

type Props = {
  url: string;
  ids: DocumentId[];
};
export function ScrapeResultCard({ url, ids }: Props) {
  const { value } = useSuspenseQuery({
    typeName: 'GetDocuments',
    ids,
  });

  return (
    <CardContainer leftToolbar={<span className="section-heading text-lg">Scrape result</span>}>
      <p className="mb-8">
        <div className="font-semibold text-sky-800 uppercase mb-2">Original url</div>
        <Link url={url}>{url}</Link>
      </p>

      <div className="font-semibold text-sky-800 uppercase mb-2">{ids.length} new documents</div>
      <div className="flex flex-col gap-2">
        {value?.documents.map((item) => (
          <Ref
            key={item.id}
            documentId={item.id}
            documentType={item.documentType}
            subtype={item.subtype}
            documentTitle={item.title}
          />
        ))}
      </div>
    </CardContainer>
  );
}
