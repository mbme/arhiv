import { DocumentType } from 'dto';
import { getDocumentTypes, isErasedDocument } from 'utils/schema';
import { Button } from 'components/Button';
import { useCardContext } from './workspace-reducer';
import { CardContainer } from './CardContainer';

export function BrowserCard() {
  const context = useCardContext();

  const openCatalog = (documentType?: DocumentType) => {
    context.replace({ variant: 'catalog', documentType });
  };

  return (
    <CardContainer>
      <CardContainer.Topbar
        left={<span className="section-heading text-lg">Browser</span>}
        right={<CardContainer.CloseButton />}
      />

      <div className="flex justify-around mt-8">
        <section>
          <h1 className="section-heading ml-4">Documents</h1>
          {getDocumentTypes(false).map((documentType) => {
            if (isErasedDocument(documentType)) {
              return null;
            }

            return (
              <Button key={documentType} variant="simple" onClick={() => openCatalog(documentType)}>
                {documentType}
              </Button>
            );
          })}

          <Button
            key=""
            variant="simple"
            onClick={() => openCatalog()}
            className="mt-4 line-through"
          >
            ERASED
          </Button>
        </section>

        <section>
          <h1 className="section-heading ml-4">Collections</h1>
          {getDocumentTypes(true).map((documentType) => (
            <Button key={documentType} variant="simple" onClick={() => openCatalog(documentType)}>
              {documentType}
            </Button>
          ))}
        </section>
      </div>
    </CardContainer>
  );
}
