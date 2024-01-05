import { DataSchema } from 'utils/schema';

type Features = {
  scraper: boolean;
};

declare global {
  interface Window {
    BASE_PATH: string;
    SCHEMA: DataSchema;
    FEATURES: Features;
  }
}
