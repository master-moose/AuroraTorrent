import { invoke } from '@tauri-apps/api/tauri';

export const sendRpc = async (method: string, params: any = {}) => {
    const request: any = {
        jsonrpc: "2.0",
        id: Date.now(),
        method,
    };

    if (Object.keys(params).length > 0) {
        request.params = params;
    }

    try {
        const response = await invoke('rpc_request', { request: JSON.stringify(request) });
        return JSON.parse(response as string);
    } catch (e) {
        console.error("RPC Error:", e);
        return null;
    }
};
