import { createRPCProxy } from '../scripts/rpc';
import { WorkspaceRequest, WorkspaceResponse } from './dto';

export const RPC = createRPCProxy<WorkspaceRequest, WorkspaceResponse>('/workspace_api');
