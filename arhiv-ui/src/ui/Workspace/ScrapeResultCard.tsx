import { DocumentId } from 'dto';
import { RefListContainer } from 'components/Ref';
import { Link } from 'components/Link';
import { CardContainer } from './CardContainer';
import { useCardContext } from './workspace-reducer';

type Props = {
  url: string;
  ids: DocumentId[];
};
export function ScrapeResultCard({ url, ids }: Props) {
  const { open } = useCardContext();

  return (
    <CardContainer>
      <CardContainer.Topbar
        left={<span className="section-heading text-lg">Scrape result</span>}
        right={<CardContainer.CloseButton />}
      />

      <p className="mb-8">
        <div className="font-semibold text-sky-800 uppercase mb-2">Original url</div>
        <Link url={url} description={url} />
      </p>

      <div className="font-semibold text-sky-800 uppercase mb-2">{ids.length} new documents</div>
      <div className="flex flex-col gap-2">
        <RefListContainer
          ids={ids}
          onClick={(documentId) => open({ variant: 'document', documentId })}
        />
      </div>
    </CardContainer>
  );
}
