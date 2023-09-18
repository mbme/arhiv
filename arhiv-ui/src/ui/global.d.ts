import { DataSchema } from 'utils/schema';

declare global {
  interface Window {
    API_ENDPOINT: string;
    SCHEMA: DataSchema;
  }
}
