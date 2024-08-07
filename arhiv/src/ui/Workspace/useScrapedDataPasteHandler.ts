import { ScrapeResult, ScraperResultsContainer } from 'scraper';
import {
  BOOK_DOCUMENT_TYPE,
  DocumentData,
  DocumentType,
  FILM_DOCUMENT_TYPE,
  GAME_DOCUMENT_TYPE,
} from 'dto';
import { useClipboardPasteHandler } from 'utils/hooks';
import { showToast } from 'components/Toaster';

const SCRAPE_RESULTS_TYPE = 'scrape-results-container';

function tryParseScraperResultsContainer(rawValue: string): ScrapeResult[] {
  if (!rawValue.includes(SCRAPE_RESULTS_TYPE)) {
    return [];
  }

  try {
    const value = JSON.parse(rawValue) as unknown;

    if (
      !value ||
      typeof value !== 'object' ||
      !('@type' in value) ||
      value['@type'] !== SCRAPE_RESULTS_TYPE
    ) {
      return [];
    }

    return (value as ScraperResultsContainer).results;
  } catch (_e) {
    /* empty */
  }

  return [];
}

export type ScrapedData = {
  documentType: DocumentType;
  data: DocumentData;
};

function scrapeResultToScrapedData(result: ScrapeResult): ScrapedData | undefined {
  console.log('Got scrape result:', result);

  switch (result.data?.typeName) {
    case 'YakabooBook': {
      const { data } = result;

      if (data.version !== 1) {
        return;
      }

      return {
        documentType: BOOK_DOCUMENT_TYPE,
        data: {
          'cover': data.coverURL,
          'title': data.title,
          'description': data.description,
          'authors': data.authors,
          'language': data.language,
          'publication_date': data.publicationDate,
          'translators': data.translators,
          'publisher': data.publisher,
          'pages': data.pages,
        },
      };
    }
    case 'IMDBFilm': {
      const { data } = result;

      if (data.version !== 1) {
        return;
      }

      return {
        documentType: FILM_DOCUMENT_TYPE,
        data: {
          'cover': data.coverURL,
          'title': data.title,
          'release_date': data.releaseDate,
          'original_language': data.originalLanguage,
          'countries_of_origin': data.countriesOfOrigin,
          'description': data.description,
          'duration': data.duration,
          'seasons': data.seasons ?? null,
          'episodes': data.episodes ?? null,
          'creators': data.creators,
          'cast': data.cast,
        },
      };
    }
    case 'MyAnimeListAnime': {
      const { data } = result;

      if (data.version !== 1) {
        return;
      }

      return {
        documentType: FILM_DOCUMENT_TYPE,
        data: {
          'cover': data.coverURL,
          'title': data.title,
          'release_date': data.releaseDate,
          'original_language': 'Japanese',
          'countries_of_origin': 'Japan',
          'creators': data.creators,
          'duration': data.duration,
          'description': data.description,
        },
      };
    }
    case 'SteamGame': {
      const { data } = result;

      if (data.version !== 1) {
        return;
      }

      return {
        documentType: GAME_DOCUMENT_TYPE,
        data: {
          'cover': data.coverURL,
          'name': data.name,
          'release_date': data.releaseDate,
          'developers': data.developers,
          'description': data.description,
        },
      };
    }
    default:
      return;
  }
}

export function useScrapedDataPasteHandler(handler: (data: ScrapedData[]) => void) {
  useClipboardPasteHandler(async (data) => {
    const textItems = await Promise.all(
      [...data.items]
        .filter((item) => item.kind === 'string')
        .map(
          (item) =>
            new Promise<string>((resolve) => {
              item.getAsString(resolve);
            }),
        ),
    );

    const allResults = textItems.flatMap(tryParseScraperResultsContainer);

    const errorResults: ScrapeResult[] = [];
    const uncapturedResults: ScrapeResult[] = [];
    const capturedResults: ScrapedData[] = [];

    for (const scrapeResult of allResults) {
      if (scrapeResult.error) {
        errorResults.push(scrapeResult);
        continue;
      }

      const capturedData = scrapeResultToScrapedData(scrapeResult);
      if (capturedData) {
        capturedResults.push(capturedData);
      } else {
        uncapturedResults.push(scrapeResult);
      }
    }

    showToast({
      level: 'info',
      message: `Captured ${capturedResults.length} out of ${allResults.length} scraper results`,
    });

    if (uncapturedResults.length > 0) {
      console.warn('Uncaptured scraper results:', uncapturedResults);
    }

    if (errorResults.length > 0) {
      console.error('Scraper errors:', errorResults);
      showToast({
        level: 'warn',
        message: `Got ${errorResults.length} scraper errors`,
      });
    }

    handler(capturedResults);
  });
}
