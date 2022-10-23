import { createRPCProxy } from './utils/rpc';
import { WorkspaceRequest, WorkspaceResponse } from '../dto';

export const RPC = createRPCProxy<WorkspaceRequest, WorkspaceResponse>('/workspace_api');
