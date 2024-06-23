import { DataSchema } from 'utils/schema';
import { WorkspaceController } from 'Workspace/controller';

type Features = {
  scraper: boolean;
  use_local_storage: boolean;
};

declare global {
  interface Window {
    BASE_PATH: string;
    SCHEMA: DataSchema;
    FEATURES: Features;
    MIN_LOGIN_LENGTH: number;
    MIN_PASSWORD_LENGTH: number;
    CREATE_ARHIV: boolean;

    WORKSPACE: WorkspaceController;
  }
}

export {};
