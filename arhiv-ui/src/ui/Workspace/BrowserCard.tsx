import { getDocumentTypes } from 'utils/schema';
import { Button } from 'components/Button';
import { useCardContext } from './workspace-reducer';
import { CardContainer } from './CardContainer';

export function BrowserCard() {
  const context = useCardContext();

  const openCatalog = (documentType: string) => {
    context.replace({ variant: 'catalog', documentType });
  };

  return (
    <>
      <CardContainer.Topbar
        left={<span className="section-heading text-lg">Browser</span>}
        right={<CardContainer.CloseButton />}
      />

      <section className="mb-8">
        <h1 className="section-heading">Documents</h1>
        {getDocumentTypes(false).map((documentType) => (
          <Button key={documentType} variant="simple" onClick={() => openCatalog(documentType)}>
            {documentType}
          </Button>
        ))}

        <Button
          key=""
          variant="simple"
          onClick={() => openCatalog('')}
          className="mt-4 line-through"
        >
          ERASED
        </Button>
      </section>

      <section>
        <h1 className="section-heading">Collections</h1>
        {getDocumentTypes(true).map((documentType) => (
          <Button key={documentType} variant="simple" onClick={() => openCatalog(documentType)}>
            {documentType}
          </Button>
        ))}
      </section>
    </>
  );
}
